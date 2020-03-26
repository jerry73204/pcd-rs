use proc_macro2::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{
    spanned::Spanned, Attribute, Data, DeriveInput, Error as SynError, Fields, FieldsNamed, Ident,
    Lit, Meta, NestedMeta, Result as SynResult, Type, TypeArray, TypePath,
};

struct DerivedTokens {
    pub write_spec_tokens: TokenStream,
    pub bin_write_tokens: TokenStream,
    pub text_write_tokens: TokenStream,
}

pub fn f_pcd_record_write_derive(input: DeriveInput) -> SynResult<TokenStream> {
    let struct_name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(SynError::new(
            input.span(),
            "Canont derive PcdSerialize for struct with generics",
        ));
    }

    let data = match &input.data {
        Data::Struct(data) => data,
        Data::Enum(_) => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PcdSerialize for enum",
            ))
        }
        Data::Union(_) => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PcdSerialize for union",
            ))
        }
    };

    let DerivedTokens {
        write_spec_tokens,
        bin_write_tokens,
        text_write_tokens,
    } = match &data.fields {
        Fields::Named(fields) => derive_named_fields(struct_name, &fields)?,
        Fields::Unnamed(_) => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PcdSerialize for tuple struct",
            ))
        }
        Fields::Unit => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PcdSerialize for unit struct",
            ))
        }
    };

    let expanded = quote! {
        impl pcd_rs::record::PcdSerialize for #struct_name {
            fn is_dynamic() -> bool {
                false
            }

            fn write_spec() -> Vec<(String, pcd_rs::metas::ValueKind, usize)> {
                #write_spec_tokens
            }

            fn write_chunk<R: std::io::Write>(&self, writer: &mut R, _: &[(String, ValueKind, usize)]) -> pcd_rs::failure::Fallible<()> {
                use pcd_rs::byteorder::{LittleEndian, WriteBytesExt};
                { #bin_write_tokens };
                Ok(())
            }

            fn write_line<R: std::io::Write>(&self, writer: &mut R, _: &[(String, ValueKind, usize)]) -> pcd_rs::failure::Fallible<()> {
                let mut tokens = Vec::<String>::new();
                { #text_write_tokens };
                let line = tokens.join(" ");
                writeln!(writer, "{}", line)?;
                Ok(())
            }
        }
    };

    Ok(expanded)
}

fn derive_named_fields(struct_name: &Ident, fields: &FieldsNamed) -> SynResult<DerivedTokens> {
    let (field_idents, write_specs, bin_write_fields, text_write_fields) = fields
        .named
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let field_error = SynError::new(
                field.span(),
                "Type of struct field must be a primitive type or array of primitive type.",
            );
            let field_ident = format_ident!("{}", &field.ident.as_ref().unwrap());

            let rename_opt = parse_field_attributes(&field.attrs)?;
            let pcd_name = if let Some(name) = rename_opt {
                name
            } else {
                field_ident.to_string()
            };

            let tokens = match &field.ty {
                Type::Array(array) => derive_array_field(&field_ident, array).ok_or(field_error)?,
                Type::Path(path) => {
                    derive_path_field(field_index, &field_ident, path).ok_or(field_error)?
                }
                _ => return Err(field_error),
            };

            Ok((field_ident, pcd_name, tokens))
        })
        .collect::<SynResult<Vec<_>>>()?
        .into_iter()
        .fold(
            (vec![], vec![], vec![], vec![]),
            |(mut field_idents, mut write_specs, mut bin_write_fields, mut text_write_fields),
             (field_ident, pcd_name, tokens)| {
                let write_spec_tokens = tokens.write_spec_tokens;
                field_idents.push(field_ident);
                write_specs.push(quote! { (#pcd_name.to_owned(), #write_spec_tokens) });
                bin_write_fields.push(tokens.bin_write_tokens);
                text_write_fields.push(tokens.text_write_tokens);
                (
                    field_idents,
                    write_specs,
                    bin_write_fields,
                    text_write_fields,
                )
            },
        );

    let write_spec_tokens = quote! { vec![#(#write_specs),*] };
    let bin_write_tokens = quote! {
        let #struct_name { #(#field_idents),* } = self;
        #(#bin_write_fields)*
    };
    let text_write_tokens = quote! {
        let #struct_name { #(#field_idents),* } = self;
        #(#text_write_fields)*
    };

    let derived_tokens = DerivedTokens {
        write_spec_tokens,
        bin_write_tokens,
        text_write_tokens,
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
        write_spec_tokens: write_spec,
        bin_write_tokens: bin_write,
        text_write_tokens: text_write,
    } = make_rw_expr(type_ident)?;

    let write_spec_tokens = quote! { #write_spec, #len };
    let bin_write_tokens = quote! {
        for value_ref in #var_ident.iter() {
            let value = *value_ref;
            #bin_write;
        }
    };
    let text_write_tokens = quote! {
        for value_ref in #var_ident.iter() {
            let value = *value_ref;
            #text_write;
        }
    };

    let derived_tokens = DerivedTokens {
        write_spec_tokens,
        bin_write_tokens,
        text_write_tokens,
    };

    Some(derived_tokens)
}

fn derive_path_field(
    _field_index: usize,
    var_ident: &Ident,
    path: &TypePath,
) -> Option<DerivedTokens> {
    let type_ident = path.path.get_ident()?;
    derive_primitive_field(var_ident, type_ident)
}

fn derive_primitive_field(var_ident: &Ident, type_ident: &Ident) -> Option<DerivedTokens> {
    let DerivedTokens {
        write_spec_tokens: write_spec,
        bin_write_tokens: bin_write,
        text_write_tokens: text_write,
    } = make_rw_expr(type_ident)?;

    let write_spec_tokens = quote! { #write_spec, 1 };
    let bin_write_tokens = quote! {
        {
            let value = *#var_ident;
            #bin_write;
        }
    };
    let text_write_tokens = quote! {
        {
            let value = *#var_ident;
            #text_write;
        }
    };

    let derived_tokens = DerivedTokens {
        write_spec_tokens,
        bin_write_tokens,
        text_write_tokens,
    };

    Some(derived_tokens)
}

fn make_rw_expr(type_ident: &Ident) -> Option<DerivedTokens> {
    let (write_spec_tokens, bin_write_tokens, text_write_tokens) =
        match type_ident.to_string().as_str() {
            "u8" => (
                quote! { pcd_rs::metas::ValueKind::U8 },
                quote! { writer.write_u8(value)? },
                quote! { tokens.push(u8::to_string(&value)) },
            ),
            "u16" => (
                quote! { pcd_rs::metas::ValueKind::U16 },
                quote! { writer.write_u16::<LittleEndian>(value)? },
                quote! { tokens.push(u16::to_string(&value)) },
            ),
            "u32" => (
                quote! { pcd_rs::metas::ValueKind::U32 },
                quote! { writer.write_u32::<LittleEndian>(value)? },
                quote! { tokens.push(u32::to_string(&value)) },
            ),
            "i8" => (
                quote! { pcd_rs::metas::ValueKind::I8 },
                quote! { writer.write_i8(value)? },
                quote! { tokens.push(i8::to_string(&value)) },
            ),
            "i16" => (
                quote! { pcd_rs::metas::ValueKind::I16 },
                quote! { writer.write_i16::<LittleEndian>(value)? },
                quote! { tokens.push(i16::to_string(&value)) },
            ),
            "i32" => (
                quote! { pcd_rs::metas::ValueKind::I32 },
                quote! { writer.write_i32::<LittleEndian>(value)? },
                quote! { tokens.push(i32::to_string(&value)) },
            ),
            "f32" => (
                quote! { pcd_rs::metas::ValueKind::F32 },
                quote! { writer.write_f32::<LittleEndian>(value)? },
                quote! { tokens.push(f32::to_string(&value)) },
            ),
            "f64" => (
                quote! { pcd_rs::metas::ValueKind::F64 },
                quote! { writer.write_f64::<LittleEndian>(value)? },
                quote! { tokens.push(f64::to_string(&value)) },
            ),
            _ => return None,
        };

    let derived_tokens = DerivedTokens {
        write_spec_tokens,
        bin_write_tokens,
        text_write_tokens,
    };

    Some(derived_tokens)
}

fn parse_field_attributes(attrs: &[Attribute]) -> SynResult<Option<String>> {
    let name_regex = Regex::new(r"^[[:word:]]+$").unwrap();

    let rename_opt = attrs
        .iter()
        .filter_map(|attr| {
            let attr_ident = attr.path.get_ident()?;
            Some((attr, attr_ident))
        })
        .fold(Ok(None), |result, (attr, attr_ident)| -> SynResult<_> {
            let name_opt = result?;
            let attr_ident_name = attr_ident.to_string();

            match attr_ident_name.as_str() {
                "pcd_rename" => {
                    if let Some(_) = name_opt {
                        let error = SynError::new(
                            attr.span(),
                            "\"pcd_rename\" cannot be specified more than once.",
                        );
                        return Err(error.into());
                    }

                    let format_error = SynError::new(
                        attr.span(),
                        "The attribute must be in form of #[pcd_rename(\"...\")].",
                    );
                    let name = match attr.parse_meta()? {
                        Meta::List(meta_list) => {
                            if meta_list.nested.len() != 1 {
                                return Err(format_error.into());
                            }

                            let nested = &meta_list.nested[0];

                            if let NestedMeta::Lit(Lit::Str(litstr)) = nested {
                                let name = litstr.value();
                                let error = SynError::new(
                                    litstr.span(),
                                    "The name argument must be composed of word characters.",
                                );
                                name_regex.find(&name).ok_or(error)?;
                                name
                            } else {
                                return Err(format_error.into());
                            }
                        }
                        _ => return Err(format_error.into()),
                    };

                    Ok(Some(name))
                }
                _ => Ok(name_opt),
            }
        })?;

    Ok(rename_opt)
}
