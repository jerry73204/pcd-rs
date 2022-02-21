use crate::{common::*, parse::ItemStruct, utils::parse_field_attributes};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token, Field, Ident, Type, TypeArray, TypePath,
};

struct DerivedTokens {
    pub write_spec_tokens: TokenStream,
    pub bin_write_tokens: TokenStream,
    pub text_write_tokens: TokenStream,
}

pub fn f_pcd_record_write_derive(item: ItemStruct) -> syn::Result<TokenStream> {
    let struct_name = &item.ident;

    let DerivedTokens {
        write_spec_tokens,
        bin_write_tokens,
        text_write_tokens,
    } = derive_named_fields(struct_name, &item.fields)?;

    let expanded = quote! {
        impl ::pcd_rs::record::PcdSerialize for #struct_name {
            fn is_dynamic() -> bool {
                false
            }

            fn write_spec() -> ::pcd_rs::metas::Schema {
                #write_spec_tokens
            }

            fn write_chunk<R: std::io::Write>(&self, writer: &mut R, _: &::pcd_rs::metas::Schema) -> ::pcd_rs::anyhow::Result<()> {
                use ::pcd_rs::byteorder::{LittleEndian, WriteBytesExt};
                { #bin_write_tokens };
                Ok(())
            }

            fn write_line<R: std::io::Write>(&self, writer: &mut R, _: &::pcd_rs::metas::Schema) -> ::pcd_rs::anyhow::Result<()> {
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

fn derive_named_fields(
    struct_name: &Ident,
    fields: &Punctuated<Field, token::Comma>,
) -> syn::Result<DerivedTokens> {
    let fields: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let field_error = syn::Error::new(
                field.span(),
                "Type of struct field must be a primitive type or array of primitive type.",
            );
            let field_ident = format_ident!("{}", &field.ident.as_ref().unwrap());

            let pcd_name = {
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

            Ok((field_ident, pcd_name, tokens))
        })
        .try_collect()?;

    let (field_idents, write_specs, bin_write_fields, text_write_fields) = fields
        .into_iter()
        .map(|(field_ident, pcd_name, tokens)| {
            let write_spec_tokens = tokens.write_spec_tokens;
            (
                field_ident,
                quote! { (#pcd_name.to_owned(), #write_spec_tokens) },
                tokens.bin_write_tokens,
                tokens.text_write_tokens,
            )
        })
        .unzip_n_vec();

    let write_spec_tokens = quote! {
        vec![#(#write_specs),*]
            .into_iter()
            .collect::<::pcd_rs::metas::Schema>()
    };
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
                quote! { ::pcd_rs::metas::ValueKind::U8 },
                quote! { writer.write_u8(value)? },
                quote! { tokens.push(u8::to_string(&value)) },
            ),
            "u16" => (
                quote! { ::pcd_rs::metas::ValueKind::U16 },
                quote! { writer.write_u16::<LittleEndian>(value)? },
                quote! { tokens.push(u16::to_string(&value)) },
            ),
            "u32" => (
                quote! { ::pcd_rs::metas::ValueKind::U32 },
                quote! { writer.write_u32::<LittleEndian>(value)? },
                quote! { tokens.push(u32::to_string(&value)) },
            ),
            "i8" => (
                quote! { ::pcd_rs::metas::ValueKind::I8 },
                quote! { writer.write_i8(value)? },
                quote! { tokens.push(i8::to_string(&value)) },
            ),
            "i16" => (
                quote! { ::pcd_rs::metas::ValueKind::I16 },
                quote! { writer.write_i16::<LittleEndian>(value)? },
                quote! { tokens.push(i16::to_string(&value)) },
            ),
            "i32" => (
                quote! { ::pcd_rs::metas::ValueKind::I32 },
                quote! { writer.write_i32::<LittleEndian>(value)? },
                quote! { tokens.push(i32::to_string(&value)) },
            ),
            "f32" => (
                quote! { ::pcd_rs::metas::ValueKind::F32 },
                quote! { writer.write_f32::<LittleEndian>(value)? },
                quote! { tokens.push(f32::to_string(&value)) },
            ),
            "f64" => (
                quote! { ::pcd_rs::metas::ValueKind::F64 },
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
