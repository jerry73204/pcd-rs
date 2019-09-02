use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    spanned::Spanned, Data, DeriveInput, Error as SynError, Fields, FieldsNamed, FieldsUnnamed,
    GenericArgument, Ident, PathArguments, Result as SynResult, Type, TypeArray, TypePath,
};

enum FieldKind {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
}

struct FieldSpec {
    kind: FieldKind,
    count: TokenStream2,
}

pub fn f_pcd_record_derive(input: DeriveInput) -> SynResult<TokenStream> {
    let struct_name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(SynError::new(
            input.span(),
            "Canont derive PCDRecord for struct with generics",
        ));
    }

    let data = match &input.data {
        Data::Struct(data) => data,
        Data::Enum(_) => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PCDRecord for enum",
            ))
        }
        Data::Union(_) => {
            return Err(SynError::new(
                input.span(),
                "Canont derive PCDRecord for union",
            ))
        }
    };

    let (record_spec_body, bin_read_body, bin_write_body, text_read_body, text_write_body) =
        match &data.fields {
            Fields::Named(fields) => derive_named_fields(struct_name, &fields)?,
            Fields::Unnamed(fields) => derive_unnamed_fields(struct_name, &fields)?,
            Fields::Unit => {
                return Err(SynError::new(
                    input.span(),
                    "Canont derive PCDRecord for unit struct",
                ))
            }
        };

    let expanded = quote! {
        impl PCDRecord for #struct_name {
            fn record_spec() -> Vec<(pcd_rs::ValueKind, Option<usize>)> {
                #record_spec_body
            }

            fn read_chunk<R: std::io::BufRead>(reader: &mut R, field_defs: &[pcd_rs::FieldDef]) -> pcd_rs::failure::Fallible<#struct_name> {
                use pcd_rs::byteorder::{LittleEndian, ReadBytesExt};
                let result = { #bin_read_body };
                Ok(result)
            }

            fn write_chunk<R: std::io::Write>(&self, writer: &mut R, field_defs: &[pcd_rs::FieldDef]) -> pcd_rs::failure::Fallible<()> {
                use pcd_rs::byteorder::{LittleEndian, WriteBytesExt};
                { #bin_write_body };
                Ok(())
            }

            fn read_line<R: std::io::BufRead>(reader: &mut R, field_defs: &[pcd_rs::FieldDef]) -> pcd_rs::failure::Fallible<#struct_name> {
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

                let result = { #text_read_body };
                Ok(result)
            }

            fn write_line<R: std::io::Write>(&self, writer: &mut R, field_defs: &[pcd_rs::FieldDef]) -> pcd_rs::failure::Fallible<()> {
                // use pcd_rs::byteorder::{LittleEndian, WriteBytesExt};
                let mut tokens = Vec::<String>::new();
                { #text_write_body };
                let line = tokens.join(" ");
                writeln!(writer, "{}", line)?;
                Ok(())
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

fn derive_named_fields(
    struct_name: &Ident,
    fields: &FieldsNamed,
) -> SynResult<(
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
)> {
    let (field_idents, field_specs, bin_read_fields, bin_write_fields, text_read_fields, text_write_fields) = fields
        .named
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let field_error = SynError::new(
                field.span(),
                "Type of struct field must be a primitive type, array of primitive type, or Vec<_> of primitive type",
            );

            let field_ident = format_ident!("{}", &field.ident.as_ref().unwrap());

            let (field_spec, bin_read_field, bin_write_field, text_read_field, text_write_field) = match &field.ty {
                Type::Array(array) => {
                    match derive_array_field(&field_ident, array) {
                        Some(result) => result,
                        None => return Err(field_error),
                    }
                }
                Type::Path(path) => {
                    match derive_path_field(field_index, &field_ident, path) {
                        Some(result) => result,
                        None => return Err(field_error),
                    }
                }
                _ => {
                    return Err(field_error)
                }
            };

            Ok((field_ident, field_spec, bin_read_field, bin_write_field, text_read_field, text_write_field))
        })
        .collect::<SynResult<Vec<_>>>()?
        .into_iter()
        .fold(
            (vec![], vec![], vec![], vec![], vec![], vec![]),
            |(mut field_idents, mut field_specs, mut bin_read_fields, mut bin_write_fields, mut text_read_fields, mut text_write_fields), (field_ident, field_spec, bin_read_field, bin_write_field, text_read_field, text_write_field)| {
                field_idents.push(field_ident);
                field_specs.push(field_spec);
                bin_read_fields.push(bin_read_field);
                bin_write_fields.push(bin_write_field);
                text_read_fields.push(text_read_field);
                text_write_fields.push(text_write_field);
                (field_idents, field_specs, bin_read_fields, bin_write_fields, text_read_fields, text_write_fields)
            }
        );

    let field_spec_tuples = field_specs
        .into_iter()
        .map(|spec| {
            use FieldKind::*;

            let kind = match spec.kind {
                U8 => quote! { pcd_rs::ValueKind::U8 },
                U16 => quote! { pcd_rs::ValueKind::U16 },
                U32 => quote! { pcd_rs::ValueKind::U32 },
                I8 => quote! { pcd_rs::ValueKind::I8 },
                I16 => quote! { pcd_rs::ValueKind::I16 },
                I32 => quote! { pcd_rs::ValueKind::I32 },
                F32 => quote! { pcd_rs::ValueKind::F32 },
                F64 => quote! { pcd_rs::ValueKind::F64 },
            };
            let count = spec.count;

            quote! { (#kind, #count) }
        })
        .collect::<Vec<_>>();

    let record_spec_body = quote! {
        vec![#(#field_spec_tuples),*]
    };

    let bin_read_body = quote! {
        #(#bin_read_fields)*

        #struct_name {
            #(#field_idents),*
        }
    };

    let bin_write_body = quote! {
        let #struct_name { #(#field_idents),* } = self;
        #(#bin_write_fields)*
    };

    let text_read_body = quote! {
        #(#text_read_fields)*

        #struct_name {
            #(#field_idents),*
        }
    };

    let text_write_body = quote! {
        let #struct_name { #(#field_idents),* } = self;
        #(#text_write_fields)*
    };

    Ok((
        record_spec_body,
        bin_read_body,
        bin_write_body,
        text_read_body,
        text_write_body,
    ))
}

fn derive_unnamed_fields(
    struct_name: &Ident,
    fields: &FieldsUnnamed,
) -> SynResult<(
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
)> {
    let (var_idents, field_specs, bin_read_fields, bin_write_fields, text_read_fields, text_write_fields) = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let field_error = SynError::new(
                field.span(),
                "Type of struct field must be a primitive type, array of primitive type, or Vec<_> of primitive type",
            );

            let var_ident = format_ident!("_{}", field_index);

            let (field_spec, bin_read_field, bin_write_field, text_read_field, text_write_field) = match &field.ty {
                Type::Array(array) => {
                    match derive_array_field(&var_ident, array) {
                        Some(result) => result,
                        None => return Err(field_error),
                    }
                }
                Type::Path(path) => {
                    match derive_path_field(field_index, &var_ident, path) {
                        Some(result) => result,
                        None => return Err(field_error),
                    }
                }
                _ => {
                    return Err(field_error)
                }
            };

            Ok((var_ident, field_spec, bin_read_field, bin_write_field, text_read_field, text_write_field))
        })
        .collect::<SynResult<Vec<_>>>()?
        .into_iter()
        .fold(
            (vec![], vec![], vec![], vec![], vec![], vec![]),
            |(mut var_idents, mut field_specs, mut bin_read_fields, mut bin_write_fields, mut text_read_fields, mut text_write_fields), (var_ident, field_spec, bin_read_field, bin_write_field, text_read_field, text_write_field)| {
                var_idents.push(var_ident);
                field_specs.push(field_spec);
                bin_read_fields.push(bin_read_field);
                bin_write_fields.push(bin_write_field);
                text_read_fields.push(text_read_field);
                text_write_fields.push(text_write_field);
                (var_idents, field_specs, bin_read_fields, bin_write_fields, text_read_fields, text_write_fields)
            }
        );

    let field_spec_tuples = field_specs
        .into_iter()
        .map(|spec| {
            use FieldKind::*;

            let kind = match spec.kind {
                U8 => quote! { pcd_rs::ValueKind::U8 },
                U16 => quote! { pcd_rs::ValueKind::U16 },
                U32 => quote! { pcd_rs::ValueKind::U32 },
                I8 => quote! { pcd_rs::ValueKind::I8 },
                I16 => quote! { pcd_rs::ValueKind::I16 },
                I32 => quote! { pcd_rs::ValueKind::I32 },
                F32 => quote! { pcd_rs::ValueKind::F32 },
                F64 => quote! { pcd_rs::ValueKind::F64 },
            };
            let count = spec.count;

            quote! { (#kind, #count) }
        })
        .collect::<Vec<_>>();

    let record_spec_body = quote! {
        vec![#(#field_spec_tuples),*]
    };

    let bin_read_body = quote! {
        #(#bin_read_fields)*

        #struct_name (
            #(#var_idents),*
        )
    };

    let bin_write_body = quote! {
        let #struct_name ( #(#var_idents),* ) = self;
        #(#bin_write_fields)*
    };

    let text_read_body = quote! {
        #(#text_read_fields)*

        #struct_name (
            #(#var_idents),*
        )
    };

    let text_write_body = quote! {
        let #struct_name ( #(#var_idents),* ) = self;
        #(#text_write_fields)*
    };

    Ok((
        record_spec_body,
        bin_read_body,
        bin_write_body,
        text_read_body,
        text_write_body,
    ))
}

fn derive_array_field(
    var_ident: &Ident,
    array: &TypeArray,
) -> Option<(
    FieldSpec,
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
)> {
    let len = &array.len;

    let type_ident = if let Type::Path(path) = &*array.elem {
        if let Some(ident) = path.path.get_ident() {
            ident
        } else {
            return None;
        }
    } else {
        return None;
    };

    let (kind, bin_read, bin_write, text_read, text_write) = match make_rw_expr(type_ident) {
        Some(result) => result,
        None => return None,
    };

    let bin_read_field = {
        let init_expr = match kind {
            FieldKind::F32 | FieldKind::F64 => quote! {[0.0; #len]},
            _ => quote! {[0; #len]},
        };

        quote! {
            let mut #var_ident = #init_expr;

            for idx in 0..(#len) {
                #var_ident [idx] = { #bin_read };
            }
        }
    };

    let bin_write_field = quote! {
        for value_ref in #var_ident.iter() {
            let value = *value_ref;
            #bin_write;
        }
    };

    let text_read_field = {
        let init_expr = match kind {
            FieldKind::F32 | FieldKind::F64 => quote! {[0.0; #len]},
            _ => quote! {[0; #len]},
        };

        quote! {
            let mut #var_ident = #init_expr;

            for idx in 0..(#len) {
                #var_ident [idx] = {
                    let token = tokens.next().unwrap();
                    #text_read
                };
            }
        }
    };

    let text_write_field = quote! {
        for value_ref in #var_ident.iter() {
            let value = *value_ref;
            #text_write;
        }
    };

    let field_spec = FieldSpec {
        kind,
        count: quote! { Some(#len) },
    };

    Some((
        field_spec,
        bin_read_field,
        bin_write_field,
        text_read_field,
        text_write_field,
    ))
}

fn derive_path_field(
    field_index: usize,
    var_ident: &Ident,
    path: &TypePath,
) -> Option<(
    FieldSpec,
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
)> {
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
                    Type::Path(path) => match path.path.get_ident() {
                        Some(ident) => ident,
                        None => return None,
                    },
                    _ => return None,
                },
                _ => return None,
            };

            derive_vec_field(field_index, var_ident, arg_ident)
        }
    }
}

fn derive_primitive_field(
    var_ident: &Ident,
    type_ident: &Ident,
) -> Option<(
    FieldSpec,
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
)> {
    let (kind, bin_read, bin_write, text_read, text_write) = match make_rw_expr(type_ident) {
        Some(result) => result,
        None => return None,
    };

    let field_spec = FieldSpec {
        kind,
        count: quote! { Some(1) },
    };

    let bin_read_field = quote! {
        let #var_ident = { #bin_read };
    };

    let bin_write_field = quote! {
        {
            let value = *#var_ident;
            #bin_write;
        }
    };

    let text_read_field = quote! {
        let #var_ident = {
            let token = tokens.next().unwrap();
            #text_read
        };
    };

    let text_write_field = quote! {
        {
            let value = *#var_ident;
            #text_write;
        }
    };

    Some((
        field_spec,
        bin_read_field,
        bin_write_field,
        text_read_field,
        text_write_field,
    ))
}

fn derive_vec_field(
    field_index: usize,
    var_ident: &Ident,
    arg_ident: &Ident,
) -> Option<(
    FieldSpec,
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
)> {
    let (kind, bin_read, bin_write, text_read, text_write) = match make_rw_expr(arg_ident) {
        Some(result) => result,
        None => return None,
    };

    let field_spec = FieldSpec {
        kind,
        count: quote! { None },
    };

    let bin_read_field = quote! {
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

    let bin_write_field = quote! {
        {
            let field_name = &field_defs[#field_index].name;
            let expect = field_defs[#field_index].count as usize;
            let found = #var_ident.len();

            if expect == found {
                for value_ref in #var_ident.iter() {
                    let value = *value_ref;
                    #bin_write;
                }
            } else {
                use pcd_rs::error::PCDError;
                let error = PCDError::new_field_size_mismatch_error(
                    field_name,
                    expect,
                    found,
                );
                return Err(error.into());
            }
        }
    };

    let text_read_field = quote! {
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

    let text_write_field = quote! {
        {
            let field_name = &field_defs[#field_index].name;
            let expect = field_defs[#field_index].count as usize;
            let found = #var_ident.len();

            if expect == found {
                for value_ref in #var_ident.iter() {
                    let value = *value_ref;
                    #text_write;
                }
            } else {
                use pcd_rs::error::PCDError;
                let error = PCDError::new_field_size_mismatch_error(
                    field_name,
                    expect,
                    found,
                );
                return Err(error.into());
            }
        }
    };

    Some((
        field_spec,
        bin_read_field,
        bin_write_field,
        text_read_field,
        text_write_field,
    ))
}

fn make_rw_expr(
    type_ident: &Ident,
) -> Option<(
    FieldKind,
    TokenStream2,
    TokenStream2,
    TokenStream2,
    TokenStream2,
)> {
    let exprs = match type_ident.to_string().as_str() {
        "u8" => (
            FieldKind::U8,
            quote! { reader.read_u8()? },
            quote! { writer.write_u8(value)? },
            quote! { token.parse::<u8>()? },
            quote! { tokens.push(u8::to_string(&value)) },
        ),
        "u16" => (
            FieldKind::U16,
            quote! { reader.read_u16::<LittleEndian>()? },
            quote! { writer.write_u16::<LittleEndian>(value)? },
            quote! { token.parse::<u16>()? },
            quote! { tokens.push(u16::to_string(&value)) },
        ),
        "u32" => (
            FieldKind::U32,
            quote! { reader.read_u32::<LittleEndian>()? },
            quote! { writer.write_u32::<LittleEndian>(value)? },
            quote! { token.parse::<u32>()? },
            quote! { tokens.push(u32::to_string(&value)) },
        ),
        "i8" => (
            FieldKind::I8,
            quote! { reader.read_i8()? },
            quote! { writer.write_i8(value)? },
            quote! { token.parse::<i8>()? },
            quote! { tokens.push(i8::to_string(&value)) },
        ),
        "i16" => (
            FieldKind::I16,
            quote! { reader.read_i16::<LittleEndian>()? },
            quote! { writer.write_i16::<LittleEndian>(value)? },
            quote! { token.parse::<i16>()? },
            quote! { tokens.push(i16::to_string(&value)) },
        ),
        "i32" => (
            FieldKind::I32,
            quote! { reader.read_i32::<LittleEndian>()? },
            quote! { writer.write_i32::<LittleEndian>(value)? },
            quote! { token.parse::<i32>()? },
            quote! { tokens.push(i32::to_string(&value)) },
        ),
        "f32" => (
            FieldKind::F32,
            quote! { reader.read_f32::<LittleEndian>()? },
            quote! { writer.write_f32::<LittleEndian>(value)? },
            quote! { token.parse::<f32>()? },
            quote! { tokens.push(f32::to_string(&value)) },
        ),
        "f64" => (
            FieldKind::F64,
            quote! { reader.read_f64::<LittleEndian>()? },
            quote! { writer.write_f64::<LittleEndian>(value)? },
            quote! { token.parse::<f64>()? },
            quote! { tokens.push(f64::to_string(&value)) },
        ),
        _ => return None,
    };

    Some(exprs)
}
