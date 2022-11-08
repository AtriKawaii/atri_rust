use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::HashMap;

/// 标记一个结构体或枚举类型, 将其作为本插件的插件实例
///
///
/// ## Usage
///
///
/// ```rust
/// use atri_plugin::Plugin;
/// #[atri_plugin::plugin]
/// struct MyPlugin {
///   /*Some field*/
/// }
///
/// impl Plugin for MyPlugin {
///   /*Some impls here*/
/// }
/// ```
/// 请注意有且仅有一个实现了 [`atri_plugin::Plugin`] 的结构体或枚举可以被标记为`插件`
///

#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut attrs = HashMap::<String, TokenTree>::new();

    {
        let mut name: Option<String> = None;
        let mut value: Option<TokenTree> = None;

        let mut iter = attr.into_iter();
        while let Some(token) = iter.next() {
            match token {
                TokenTree::Group(_) => {}
                TokenTree::Ident(ident) => {
                    name = Some(ident.to_string());
                }
                TokenTree::Punct(punct) => match punct.as_char() {
                    '=' => {
                        let token = iter.next().unwrap_or_else(|| {
                            panic!("No value after key: {}", name.as_ref().expect("No key"))
                        });
                        match &token {
                            TokenTree::Ident(_) | TokenTree::Literal(_) => value = Some(token),
                            or => panic!("Unexpected value: {}", or),
                        }
                    }
                    ',' => continue,
                    _ => {}
                },
                TokenTree::Literal(_) => {}
            }

            if name.is_some() && value.is_some() {
                attrs.insert(name.take().unwrap(), value.take().unwrap());
            }
        }
    }

    let mut tree: Vec<TokenTree> = input.into_iter().collect();

    let struct_name = {
        let mut iter = tree.iter();

        let mut token = None::<&TokenTree>;
        while let Some(t) = iter.next() {
            let str = t.to_string();

            match &*str {
                "struct" | "enum" => {
                    token = iter.next();
                    break;
                }
                "union" => panic!("Union is not supported"),
                _ => {}
            }
        }
        token
            .unwrap_or_else(|| panic!("Cannot find struct or enum"))
            .clone()
    };

    tree.push(TokenTree::Punct(Punct::new('#', Spacing::Alone)));
    {
        let no_mangle = TokenTree::Ident(Ident::new("no_mangle", Span::call_site()));
        let group = TokenStream::from(no_mangle);
        tree.push(TokenTree::Group(Group::new(Delimiter::Bracket, group)));
    }
    tree.push(TokenTree::Ident(Ident::new("extern", Span::call_site())));
    tree.push(TokenTree::Literal(Literal::string("C")));
    tree.push(TokenTree::Ident(Ident::new("fn", Span::call_site())));
    tree.push(TokenTree::Ident(Ident::new("on_init", Span::call_site())));
    tree.push(TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        TokenStream::new(),
    )));
    tree.push(TokenTree::Punct(Punct::new('-', Spacing::Joint)));
    tree.push(TokenTree::Punct(Punct::new('>', Spacing::Alone)));
    tree.push(TokenTree::Ident(Ident::new(
        "atri_plugin",
        Span::call_site(),
    )));
    tree.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
    tree.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
    tree.push(TokenTree::Ident(Ident::new(
        "PluginInstance",
        Span::call_site(),
    )));
    {
        let mut group = Vec::<TokenTree>::new();
        group.push(TokenTree::Ident(Ident::new(
            "atri_plugin",
            Span::call_site(),
        )));
        group.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
        group.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
        group.push(TokenTree::Ident(Ident::new(
            "__get_instance",
            Span::call_site(),
        )));

        // param
        {
            let mut new_instance = Vec::<TokenTree>::new();
            new_instance.push(TokenTree::Punct(Punct::new('<', Spacing::Alone)));
            new_instance.push(struct_name.clone());
            new_instance.push(TokenTree::Ident(Ident::new("as", Span::call_site())));
            new_instance.push(TokenTree::Ident(Ident::new(
                "atri_plugin",
                Span::call_site(),
            )));
            new_instance.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
            new_instance.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
            new_instance.push(TokenTree::Ident(Ident::new("Plugin", Span::call_site())));
            new_instance.push(TokenTree::Punct(Punct::new('>', Spacing::Joint)));
            new_instance.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
            new_instance.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
            new_instance.push(TokenTree::Ident(Ident::new("new", Span::call_site())));
            new_instance.push(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                TokenStream::new(),
            )));
            new_instance.push(TokenTree::Punct(Punct::new(',', Spacing::Joint)));
            new_instance.push(attrs.get("name").map(TokenTree::clone).unwrap_or_else(|| {
                TokenTree::Literal(Literal::string(&{
                    let mut s = struct_name.to_string();
                    s.push_str("_plugin");
                    s
                }))
            }));
            group.push(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                TokenStream::from_iter(new_instance),
            )));
        }
        tree.push(TokenTree::Group(Group::new(
            Delimiter::Brace,
            TokenStream::from_iter(group),
        )));
    }

    TokenStream::from_iter(tree)
}
