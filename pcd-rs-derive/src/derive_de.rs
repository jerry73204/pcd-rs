use crate::{common::*, parse::ItemStruct, utils::parse_field_attributes};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token, Error, Field, GenericArgument, Ident,
    PathArguments, Type, TypeArray, TypePath,
};

struct DerivedTokens {
    pub read_spec_tokens: TokenStream,
    pub bin_read_tokens: TokenStream,
    pub text_read_tokens: TokenStream,
}

pub fn f_pcd_record_read_derive(item: ItemStruct) -> syn::Result<TokenStream> {
    let struct_name = &item.ident;

    let DerivedTokens {
        read_spec_tokens,
        bin_read_tokens,
        text_read_tokens,
    } = derive_named_fields(struct_name, &item.fields)?;

    let expanded = quote! {
        impl ::pcd_rs::record::PcdDeserialize for #struct_name {
            fn is_dynamic() -> bool {
                false
            }

            fn read_spec() -> Vec<(Option<String>, ::pcd_rs::metas::ValueKind, Option<usize>)> {
                #read_spec_tokens
            }

           fn read_chunk<R: std::io::BufRead>(reader: &mut R, field_defs: &::pcd_rs::metas::Schema) -> ::pcd_rs::anyhow::Result<#struct_name> {
                use ::pcd_rs::byteorder::{LittleEndian, ReadBytesExt};
                let result = { #bin_read_tokens };
                Ok(result)
            }

            fn read_line<R: std::io::BufRead>(reader: &mut R, field_defs: &::pcd_rs::metas::Schema) -> ::pcd_rs::anyhow::Result<#struct_name> {
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
                        use ::pcd_rs::error::Error;
                        let error = Error::new_text_token_mismatch_error(expect, found);
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

fn derive_named_fields(
    struct_name: &Ident,
    fields: &Punctuated<Field, token::Comma>,
) -> syn::Result<DerivedTokens> {
    let fields: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let field_error = Error::new(
                field.span(),
                "expect a primitive type, array of primitive type, or Vec<_> of primitive type",
            );
            let field_ident = format_ident!("{}", &field.ident.as_ref().unwrap());

            // Check #[pcd(...)] options
            let pcd_name_opt = {
                let opts = parse_field_attributes(&field.attrs)?;

                match (opts.ignore, opts.rename) {
                    (true, _) => None,
                    (false, None) => Some(field_ident.to_string()),
                    (false, Some(rename)) => Some(rename),
                }
            };

            let tokens = match &field.ty {
                Type::Array(array) => derive_array_field(&field_ident, array).ok_or(field_error)?,
                Type::Path(path) => {
                    derive_path_field(field_index, &field_ident, path).ok_or(field_error)?
                }
                _ => return Err(field_error),
            };

            Ok((field_ident, pcd_name_opt, tokens))
        })
        .try_collect()?;

    let (field_idents, read_specs, bin_read_fields, text_read_fields) = fields
        .into_iter()
        .map(|(field_ident, pcd_name_opt, tokens)| {
            let read_spec_tokens = tokens.read_spec_tokens;
            let read_spec = match pcd_name_opt {
                Some(name) => quote! { (Some(#name.to_owned()), #read_spec_tokens) },
                None => quote! { (None, #read_spec_tokens) },
            };

            (
                field_ident,
                read_spec,
                tokens.bin_read_tokens,
                tokens.text_read_tokens,
            )
        })
        .unzip_n_vec();

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
                    if seg.ident != "Vec" {
                        return None;
                    }

                    match &seg.arguments {
                        PathArguments::AngleBracketed(args) => &args.args,
                        _ => return None,
                    }
                }
                3 => {
                    // Expect std::vec::Vec<_>
                    if segments[0].ident != "Vec"
                        || segments[1].ident != "vec"
                        || segments[2].ident != "Vec"
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
                GenericArgument::Type(Type::Path(path)) => path.path.get_ident()?,
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
                .collect::<::pcd_rs::anyhow::Result<Vec<_>>>()?
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
                .collect::<::pcd_rs::anyhow::Result<Vec<_>>>()?
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
                quote! { ::pcd_rs::metas::ValueKind::U8 },
                quote! { reader.read_u8()? },
                quote! { token.parse::<u8>()? },
            ),
            "u16" => (
                quote! { ::pcd_rs::metas::ValueKind::U16 },
                quote! { reader.read_u16::<LittleEndian>()? },
                quote! { token.parse::<u16>()? },
            ),
            "u32" => (
                quote! { ::pcd_rs::metas::ValueKind::U32 },
                quote! { reader.read_u32::<LittleEndian>()? },
                quote! { token.parse::<u32>()? },
            ),
            "i8" => (
                quote! { ::pcd_rs::metas::ValueKind::I8 },
                quote! { reader.read_i8()? },
                quote! { token.parse::<i8>()? },
            ),
            "i16" => (
                quote! { ::pcd_rs::metas::ValueKind::I16 },
                quote! { reader.read_i16::<LittleEndian>()? },
                quote! { token.parse::<i16>()? },
            ),
            "i32" => (
                quote! { ::pcd_rs::metas::ValueKind::I32 },
                quote! { reader.read_i32::<LittleEndian>()? },
                quote! { token.parse::<i32>()? },
            ),
            "f32" => (
                quote! { ::pcd_rs::metas::ValueKind::F32 },
                quote! { reader.read_f32::<LittleEndian>()? },
                quote! { token.parse::<f32>()? },
            ),
            "f64" => (
                quote! { ::pcd_rs::metas::ValueKind::F64 },
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
