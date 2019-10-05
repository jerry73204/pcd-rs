use proc_macro2::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{
    spanned::Spanned, Attribute, Data, DeriveInput, Error as SynError, Fields, FieldsNamed,
    FieldsUnnamed, GenericArgument, Ident, Lit, Meta, NestedMeta, PathArguments,
    Result as SynResult, Type, TypeArray, TypePath,
};

struct DerivedTokens {
    pub read_spec_tokens: TokenStream,
    pub bin_read_tokens: TokenStream,
    pub text_read_tokens: TokenStream,
}

pub fn f_pcd_record_read_derive(input: DeriveInput) -> SynResult<TokenStream> {
    let struct_name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(SynError::new(
            input.span(),
            "Canont derive PCDRecordRead for struct with generics",
        ));
    }

    let data = match &input.data {
        Data::Struct(data) => data,
        Data::Enum(_) => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PCDRecordRead for enum",
            ))
        }
        Data::Union(_) => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PCDRecordRead for union",
            ))
        }
    };

    let DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    } = match &data.fields {
        Fields::Named(fields) => derive_named_fields(struct_name, &fields)?,
        Fields::Unnamed(fields) => derive_unnamed_fields(struct_name, &fields)?,
        Fields::Unit => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PCDRecordRead for unit struct",
            ))
        }
    };

    let expanded = quote! {
        impl pcd_rs::record::PCDRecordRead for #struct_name {
            fn read_spec() -> Vec<(Option<String>, pcd_rs::meta::ValueKind, Option<usize>)> {
                #read_spec_tokens
            }

            fn read_chunk<R: std::io::BufRead>(reader: &mut R, field_defs: &[pcd_rs::meta::FieldDef]) -> pcd_rs::failure::Fallible<#struct_name> {
                use pcd_rs::byteorder::{LittleEndian, ReadBytesExt};
                let result = { #bin_read_tokens };
                Ok(result)
            }

            fn read_line<R: std::io::BufRead>(reader: &mut R, field_defs: &[pcd_rs::meta::FieldDef]) -> pcd_rs::failure::Fallible<#struct_name> {
                let mut line = String::new();
                let mut tokens = {
                    let read_size = reader.read_line(&mut line)?;
                    let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();
                    tokens.into_iter()
                };

                {
                    let expect = field_defs.iter().fold(0, |sum, def| sum + def.count as usize);
                    let (found, _) = tokens.size_hint();
                    if expect != found {
                        use pcd_rs::error::PCDError;
                        let error = PCDError::new_text_token_mismatch_error(expect, found);
                        return Err(error.into());
                    }
                }

                let result = { #text_read_tokens };
                Ok(result)
            }
        }
    };

    Ok(expanded)
}

fn derive_named_fields(struct_name: &Ident, fields: &FieldsNamed) -> SynResult<DerivedTokens> {
    let (
        field_idents,
        read_specs,
        bin_read_fields,
        text_read_fields,
    ) = fields
        .named
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let field_error = SynError::new(
                field.span(),
                "Type of struct field must be a primitive type, array of primitive type, or Vec<_> of primitive type",
            );
            let field_ident = format_ident!("{}", &field.ident.as_ref().unwrap());

            // Check #[pcd_rename(...)] and #[pcd_ignore_name] attributes
            let pcd_name_opt = {
                let (rename_opt, ignore_name) = parse_field_attributes(&field.attrs)?;

                if ignore_name {
                    None
                } else if let Some(name) = rename_opt {
                    Some(name)
                } else {
                    Some(field_ident.to_string())
                }
            };

            let tokens = match &field.ty {
                Type::Array(array) => {
                    derive_array_field(&field_ident, array)
                        .ok_or(field_error)?
                }
                Type::Path(path) => {
                    derive_path_field(field_index, &field_ident, path)
                        .ok_or(field_error)?
                }
                _ => {
                    return Err(field_error)
                }
            };

            Ok((field_ident, pcd_name_opt, tokens))
        })
        .collect::<SynResult<Vec<_>>>()?
        .into_iter()
        .fold(
            (vec![], vec![], vec![], vec![]),
            |(mut field_idents, mut read_specs, mut bin_read_fields, mut text_read_fields), (field_ident, pcd_name_opt, tokens)| {
                let read_spec_tokens = tokens.read_spec_tokens;
                let read_spec = match pcd_name_opt {
                    Some(name) => quote!{ (Some(#name.to_owned()), #read_spec_tokens) },
                    None => quote!{ (None, #read_spec_tokens) },
                };

                field_idents.push(field_ident);
                read_specs.push(read_spec);
                bin_read_fields.push(tokens.bin_read_tokens);
                text_read_fields.push(tokens.text_read_tokens);
                (field_idents, read_specs, bin_read_fields, text_read_fields)
            }
        );

    let read_spec_tokens = quote! { vec![#(#read_specs),*] };
    let bin_read_tokens = quote! {
        #(#bin_read_fields)*

        #struct_name {
            #(#field_idents),*
        }
    };
    let text_read_tokens = quote! {
        #(#text_read_fields)*

        #struct_name {
            #(#field_idents),*
        }
    };

    let derived_tokens = DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    };
    Ok(derived_tokens)
}

fn derive_unnamed_fields(struct_name: &Ident, fields: &FieldsUnnamed) -> SynResult<DerivedTokens> {
    let (
        var_idents,
        read_specs,
        bin_read_fields,
        text_read_fields,
    ) = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let field_error = SynError::new(
                field.span(),
                "Type of struct field must be a primitive type, array of primitive type, or Vec<_> of primitive type",
            );

            let var_ident = format_ident!("_{}", field_index);

            let tokens = match &field.ty {
                Type::Array(array) => {
                    derive_array_field(&var_ident, array)
                        .ok_or(field_error)?
                }
                Type::Path(path) => {
                    derive_path_field(field_index, &var_ident, path)
                        .ok_or(field_error)?
                }
                _ => {
                    return Err(field_error)
                }
            };

            Ok((var_ident, tokens))
        })
        .collect::<SynResult<Vec<_>>>()?
        .into_iter()
        .fold(
            (vec![], vec![], vec![], vec![]),
            |(mut var_idents, mut read_specs, mut bin_read_fields, mut text_read_fields), (var_ident, tokens)| {
                let read_spec_tokens = tokens.read_spec_tokens;
                let pcd_name: Option<String> = None;

                var_idents.push(var_ident);
                read_specs.push(quote!{ (#pcd_name, #read_spec_tokens) });
                bin_read_fields.push(tokens.bin_read_tokens);
                text_read_fields.push(tokens.text_read_tokens);
                (var_idents, read_specs, bin_read_fields, text_read_fields)
            }
        );

    let read_spec_tokens = quote! { vec![#(#read_specs),*] };
    let bin_read_tokens = quote! {
        #(#bin_read_fields)*

        #struct_name (
            #(#var_idents),*
        )
    };
    let text_read_tokens = quote! {
        #(#text_read_fields)*

        #struct_name (
            #(#var_idents),*
        )
    };

    let derived_tokens = DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    };
    Ok(derived_tokens)
}

fn derive_array_field(var_ident: &Ident, array: &TypeArray) -> Option<DerivedTokens> {
    let len = &array.len;
    let type_ident = match &*array.elem {
        Type::Path(path) => path.path.get_ident()?,
        _ => return None,
    };

    let DerivedTokens {
        read_spec_tokens: read_spec,
        bin_read_tokens: bin_read,
        text_read_tokens: text_read,
    } = make_rw_expr(type_ident)?;

    let read_spec_tokens = quote! { #read_spec, Some(#len) };
    let bin_read_tokens = quote! {
        let mut #var_ident = [Default::default(); #len];

        for idx in 0..(#len) {
            #var_ident [idx] = { #bin_read };
        }
    };
    let text_read_tokens = quote! {
        let mut #var_ident = [Default::default(); #len];

        for idx in 0..(#len) {
            #var_ident [idx] = {
                let token = tokens.next().unwrap();
                #text_read
            };
        }
    };

    let derived_tokens = DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    };

    Some(derived_tokens)
}

fn derive_path_field(
    field_index: usize,
    var_ident: &Ident,
    path: &TypePath,
) -> Option<DerivedTokens> {
    match path.path.get_ident() {
        Some(type_ident) => derive_primitive_field(var_ident, type_ident),
        None => {
            let segments = path.path.segments.iter().collect::<Vec<_>>();
            let vec_args = match segments.len() {
                1 => {
                    // Expect Vec<_>
                    let seg = segments[0];
                    if seg.ident.to_string() != "Vec" {
                        return None;
                    }

                    match &seg.arguments {
                        PathArguments::AngleBracketed(args) => &args.args,
                        _ => return None,
                    }
                }
                3 => {
                    // Expect std::vec::Vec<_>
                    if segments[0].ident.to_string() != "Vec"
                        || segments[1].ident.to_string() != "vec"
                        || segments[2].ident.to_string() != "Vec"
                    {
                        return None;
                    }

                    match &segments[2].arguments {
                        PathArguments::AngleBracketed(args) => &args.args,
                        _ => return None,
                    }
                }
                _ => {
                    return None;
                }
            };

            if vec_args.len() != 1 {
                return None;
            }

            let arg_ident = match &vec_args[0] {
                GenericArgument::Type(ty) => match ty {
                    Type::Path(path) => path.path.get_ident()?,
                    _ => return None,
                },
                _ => return None,
            };

            derive_vec_field(field_index, var_ident, arg_ident)
        }
    }
}

fn derive_primitive_field(var_ident: &Ident, type_ident: &Ident) -> Option<DerivedTokens> {
    let DerivedTokens {
        read_spec_tokens: read_spec,
        bin_read_tokens: bin_read,
        text_read_tokens: text_read,
    } = make_rw_expr(type_ident)?;

    let read_spec_tokens = quote! { #read_spec, Some(1) };
    let bin_read_tokens = quote! {
        let #var_ident = { #bin_read };
    };
    let text_read_tokens = quote! {
        let #var_ident = {
            let token = tokens.next().unwrap();
            #text_read
        };
    };

    let derived_tokens = DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    };

    Some(derived_tokens)
}

fn derive_vec_field(
    field_index: usize,
    var_ident: &Ident,
    arg_ident: &Ident,
) -> Option<DerivedTokens> {
    let DerivedTokens {
        read_spec_tokens: read_spec,
        bin_read_tokens: bin_read,
        text_read_tokens: text_read,
    } = make_rw_expr(arg_ident)?;

    let read_spec_tokens = quote! { #read_spec, None };
    let bin_read_tokens = quote! {
        let #var_ident = {
            let count = field_defs[#field_index].count as usize;
            (0..count)
                .into_iter()
                .map(|_| {
                    let value = { #bin_read };
                    Ok(value)
                })
                .collect::<pcd_rs::failure::Fallible<Vec<_>>>()?
        };
    };
    let text_read_tokens = quote! {
        let #var_ident = {
            let count = field_defs[#field_index].count as usize;
            (0..count)
                .into_iter()
                .map(|_| {
                    let token = tokens.next().unwrap();
                    let value = { #text_read };
                    Ok(value)
                })
                .collect::<pcd_rs::failure::Fallible<Vec<_>>>()?
        };
    };

    let derived_tokens = DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    };

    Some(derived_tokens)
}

fn make_rw_expr(type_ident: &Ident) -> Option<DerivedTokens> {
    let (read_spec_tokens, bin_read_tokens, text_read_tokens) =
        match type_ident.to_string().as_str() {
            "u8" => (
                quote! { pcd_rs::meta::ValueKind::U8 },
                quote! { reader.read_u8()? },
                quote! { token.parse::<u8>()? },
            ),
            "u16" => (
                quote! { pcd_rs::meta::ValueKind::U16 },
                quote! { reader.read_u16::<LittleEndian>()? },
                quote! { token.parse::<u16>()? },
            ),
            "u32" => (
                quote! { pcd_rs::meta::ValueKind::U32 },
                quote! { reader.read_u32::<LittleEndian>()? },
                quote! { token.parse::<u32>()? },
            ),
            "i8" => (
                quote! { pcd_rs::meta::ValueKind::I8 },
                quote! { reader.read_i8()? },
                quote! { token.parse::<i8>()? },
            ),
            "i16" => (
                quote! { pcd_rs::meta::ValueKind::I16 },
                quote! { reader.read_i16::<LittleEndian>()? },
                quote! { token.parse::<i16>()? },
            ),
            "i32" => (
                quote! { pcd_rs::meta::ValueKind::I32 },
                quote! { reader.read_i32::<LittleEndian>()? },
                quote! { token.parse::<i32>()? },
            ),
            "f32" => (
                quote! { pcd_rs::meta::ValueKind::F32 },
                quote! { reader.read_f32::<LittleEndian>()? },
                quote! { token.parse::<f32>()? },
            ),
            "f64" => (
                quote! { pcd_rs::meta::ValueKind::F64 },
                quote! { reader.read_f64::<LittleEndian>()? },
                quote! { token.parse::<f64>()? },
            ),
            _ => return None,
        };

    let derived_tokens = DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    };

    Some(derived_tokens)
}

fn parse_field_attributes(attrs: &[Attribute]) -> SynResult<(Option<String>, bool)> {
    let name_regex = Regex::new(r"^[[:word:]]+$").unwrap();

    let (rename_opt, ignore_name) = attrs
        .iter()
        .filter_map(|attr| {
            let attr_ident = attr.path.get_ident()?;
            Some((attr, attr_ident))
        } )
        .fold(
            Ok((None, false)),
            |result, (attr, attr_ident)| -> SynResult<_> {
                let (name_opt, is_ignored) = result?;

                let attr_ident_name = attr_ident.to_string();
                match attr_ident_name.as_str() {
                    "pcd_rename" => {
                        if let Some(_) = name_opt {
                            let error = SynError::new(attr.span(), "\"pcd_rename\" cannot be specified more than once.");
                            return Err(error.into());
                        }

                        if is_ignored {
                            let error = SynError::new(attr.span(), "\"pcd_rename\" and \"pcd_ignore_name\" attributes cannot appear simultaneously.");
                            return Err(error.into());
                        }

                        let format_error = SynError::new(attr.span(), "The attribute must be in form of #[pcd_rename(\"...\")].");
                        let name = match attr.parse_meta()? {
                            Meta::List(meta_list) => {
                                if meta_list.nested.len() != 1 {
                                    return Err(format_error.into());
                                }

                                let nested = &meta_list.nested[0];

                                if let NestedMeta::Lit(Lit::Str(litstr)) = nested {
                                    let name = litstr.value();
                                    let error = SynError::new(litstr.span(), "The name argument must be composed of word characters.");
                                    name_regex.find(&name).ok_or(error)?;
                                    name
                                } else {
                                    return Err(format_error.into());
                                }
                            }
                            _ => return Err(format_error.into()),
                        };

                        Ok((Some(name), false))
                    }
                    "pcd_ignore_name" => {
                        if let Some(_) = name_opt {
                            let error = SynError::new(attr.span(), "\"pcd_rename\" and pcd_\"ignore_name\" attributes cannot appear simultaneously.");
                            return Err(error.into());
                        }

                        if is_ignored {
                            let error = SynError::new(attr.span(), "\"pcd_ignore_name\" cannot be specified more than once.");
                            return Err(error.into());
                        }

                        Ok((None, true))
                    }
                    _ => Ok((name_opt, is_ignored)),
                }
            }
        )?;

    Ok((rename_opt, ignore_name))
}
