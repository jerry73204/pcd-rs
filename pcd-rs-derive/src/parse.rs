use once_cell::sync::Lazy;
use regex::Regex;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, Error, Field, Ident, LitStr, Result, Token, Visibility,
};

pub struct ItemStruct {
    pub attrs: Vec<Attribute>,
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
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse_named)?,
        })
    }
}

pub struct AttrList {
    pub paren_token: token::Paren,
    pub options: Punctuated<AttrOption, Token![,]>,
}

impl Parse for AttrList {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            paren_token: parenthesized!(content in input),
            options: content.parse_terminated(AttrOption::parse)?,
        })
    }
}

pub enum AttrOption {
    Rename(RenameAttr),
    Ignore(IgnoreAttr),
}

impl AttrOption {
    pub fn as_rename(&self) -> Option<&RenameAttr> {
        if let Self::Rename(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_ignore(&self) -> Option<&IgnoreAttr> {
        if let Self::Ignore(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl From<RenameAttr> for AttrOption {
    fn from(v: RenameAttr) -> Self {
        Self::Rename(v)
    }
}

impl From<IgnoreAttr> for AttrOption {
    fn from(v: IgnoreAttr) -> Self {
        Self::Ignore(v)
    }
}

pub struct RenameAttr {
    pub ident: Ident,
    pub eq_token: Token![=],
    pub lit: LitStr,
    pub rename: String,
}

pub struct IgnoreAttr {
    pub ident: Ident,
}

impl Parse for AttrOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;

        let attr: Self = match ident.to_string().as_str() {
            "rename" => {
                static NAME_REGEX: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"^[[:word:]]+$").unwrap());

                let eq_token = input.parse()?;
                let lit: LitStr = input.parse()?;
                let rename = lit.value();

                NAME_REGEX
                    .find(&rename)
                    .ok_or_else(|| Error::new(lit.span(), "invalid name"))?;

                RenameAttr {
                    ident,
                    eq_token,
                    lit,
                    rename,
                }
                .into()
            }
            "ignore" => IgnoreAttr { ident }.into(),
            name => {
                return Err(Error::new(
                    ident.span(),
                    format!("invalid attribute '{}'", name),
                ));
            }
        };

        Ok(attr)
    }
}
