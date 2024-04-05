use crate::common::*;
use syn::{spanned::Spanned, AttrStyle, Attribute, Error};

use crate::parse::{AttrList, AttrOption};

pub fn parse_field_attributes(attrs: &[Attribute]) -> syn::Result<Options> {
    {
        let options: Vec<_> = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("pcd"))
            .map(|attr| {
                if attr.style == AttrStyle::Outer {
                    Ok(attr)
                } else {
                    Err(Error::new(
                        attr.span(),
                        "inner pcd attribute is not supported",
                    ))
                }
            })
            .map(|attr| -> syn::Result<_> {
                let attr = attr?;
                let attr_list: AttrList = attr.parse_args()?;
                Ok(attr_list)
            })
            .try_collect()?;
        let options: Vec<AttrOption> = options.into_iter().flat_map(|list| list.options).collect();

        let ignore_option = {
            let mut ignore_opts = options.iter().filter_map(|opt| opt.as_ignore()).fuse();
            let ignore_opt = ignore_opts.next();
            if let Some(opt) = ignore_opts.next() {
                return Err(syn::Error::new(
                    opt.ident.span(),
                    "ignore option cannot specified more than once",
                ));
            }
            ignore_opt
        };
        let rename_option = {
            let mut rename_opts = options.iter().filter_map(|opt| opt.as_rename()).fuse();
            let rename_opt = rename_opts.next();
            if let Some(opt) = rename_opts.next() {
                return Err(syn::Error::new(
                    opt.ident.span(),
                    "rename option cannot specified more than once",
                ));
            }
            rename_opt
        };

        Ok(Options {
            ignore: ignore_option.is_some(),
            rename: rename_option.map(|opt| opt.rename.clone()),
        })
    }
}

pub struct Options {
    pub ignore: bool,
    pub rename: Option<String>,
}
