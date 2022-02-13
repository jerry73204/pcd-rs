use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Field, Ident, Result, Token, Visibility,
};

pub struct ItemStruct {
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

impl Parse for ItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ItemStruct {
            vis: input.parse()?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse_named)?,
        })
    }
}
