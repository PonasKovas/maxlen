use encoding::Encoding;
use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use proc_macro2::Span;
use quote::quote;
use syn::{
	Ident, LitInt, LitStr, Token,
	parse::{Parse, ParseStream},
	parse_macro_input,
};

mod encoding;

struct BStrInput {
	max: usize,
	encoding: Ident,
	str: LitStr,
}

impl Parse for BStrInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let max: LitInt = input.parse()?;
		let max = max.base10_parse()?;
		input.parse::<Token![,]>()?;

		// Look ahead to determine if encoding specified
		let lookahead = input.lookahead1();
		let encoding = if lookahead.peek(Ident) {
			// with encoding
			let ident: Ident = input.parse()?;
			input.parse::<Token![,]>()?;
			ident
		} else if lookahead.peek(LitStr) {
			// no encoding
			Ident::new("Utf8", Span::mixed_site())
		} else {
			return Err(lookahead.error());
		};

		let str: LitStr = input.parse()?;

		Ok(Self { max, encoding, str })
	}
}

#[proc_macro_error]
#[proc_macro]
pub fn bstr(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as BStrInput);

	let length = match input.encoding.to_string().as_str() {
		"Utf8" => encoding::Utf8::length,
		"Cesu8" => encoding::Cesu8::length,
		"MCesu8" => encoding::MCesu8::length,
		other => {
			abort_call_site!("Unknown encoding {:?}", other);
		}
	};

	if length(&input.str.value()) > input.max {
		abort_call_site!("Length exceeded! Max length {}", input.max);
	}

	let str = input.str;
	let encoding = input.encoding;
	let max = input.max;
	quote! {
		unsafe {
			::maxlen::BStr::<#max, ::maxlen::encoding::#encoding>::from_str_unchecked(#str)
		}
	}
	.into()
}
