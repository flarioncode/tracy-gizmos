#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

//! A procedural macro attribute for instrumenting functions with
//! [`tracy-gizmos`] zones.
//!
//! ## Usage
//!
//! In the `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tracy-gizmos-attributes = "0.0.1"
//! ```
//!
//! The [`#[instrument]`][instrument] attribute can now be added to a
//! function to automatically create and enter a `tracy-gizmos` [zone]
//! when that function is called. For example:
//!
//! ```no_run
//! #[tracy_gizmos_attributes::instrument]
//! fn work() {
//!     // do stuff
//! }
//! ```
//!
//! [`tracy-gizmos`]: https://crates.io/crates/tracy-gizmos
//! [zone]: https://docs.rs/tracy-gizmos/latest/tracy_gizmos/struct.Zone.html
//! [instrument]: macro@self::instrument

use proc_macro::{
	TokenStream,
	TokenTree,
	Span,
	Delimiter,
	Spacing,
	Group,
	Ident,
	Literal,
	Punct,
};

/// Instruments a function to create and enter a zone every time the
/// function is called.
///
/// The generated zone's name will be the name of the function.
///
/// ## Examples
///
/// ```
/// # use tracy_gizmos_attributes::instrument;
/// #[instrument]
/// fn work() {
///    // do stuff
/// }
/// ```
///
/// `const fn` cannot be instrumented, and will result in a compilation
/// failure:
///
/// ```compile_fail
/// # use tracy_gizmos_attributes::instrument;
/// #[instrument]
/// const fn work() {
///    // do stuff
/// }
/// ```
///
/// `async fn` cannot be instrumented, *yet*, and will result in a
/// compilation failure:
///
/// ```compile_fail
/// # use tracy_gizmos_attributes::instrument
/// #[instrument]
/// async fn work() {
///    // do stuff
/// }
/// ```
#[proc_macro_attribute]
pub fn instrument(_attr: TokenStream, item: TokenStream) -> TokenStream {
	// Cloning a `TokenStream` is cheap since it's reference counted
	// internally.
	let instrumented = try_instrument(item.clone());
	// We chain both error and original item, to prevent the
	// generation of two compilation errors: one from us and another
	// one (or multiple, even) caused by original item being skipped.
	match instrumented {
		Ok(item) => item,
		Err(e)   => TokenStream::from_iter(e.to_compile_error().into_iter().chain(item)),
	}
}

fn try_instrument(item: TokenStream) -> Result<TokenStream, Error> {
	// Function item's grammar:
	// https://doc.rust-lang.org/reference/items/functions.html
	// Put simply, it boils down to:
	// ... const? async? fn $name:ident ... {}?

	let mut tokens: Vec<TokenTree> = item.into_iter().collect();

	let mut tokens_it = tokens.iter();

	for t in tokens_it.by_ref() {
		if let TokenTree::Ident(i) = t {
			match i.to_string().as_str() {
				"const" => return Err(Error::new("Const functions can't be instrumented.", t.span())),
				// Could be supported when fibers are implemented. Then, we can
				// just generate a fiber-zone or whatever.
				"async" => return Err(Error::new("Async functions can't be instrumented, yet.", t.span())),
				"fn"    => break,
				_       => continue,
			}
		}
	}

	// Here, either iterator is empty now or we've just consumed the
	// `fn` and ready to get the function name.

	let Some(TokenTree::Ident(i)) = tokens_it.next() else {
		let span = tokens.first().unwrap().span();
		return Err(Error::new("Only functions can be instrumented.", span));
	};

	let name = i.to_string();
	// r# is only important for the rustc, Tracy zone name can be
	// whatever.
	let name = name.strip_prefix("r#").unwrap_or(&name);

	// The function body should be the last token tree.
	let body = match tokens.pop() {
		Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g,
		Some(t) => return Err(Error::new("Function without a body can't be instrumented.", t.span())),
		_ => unreachable!(),
	};

	let instrumented_body = vec![make_our_zone(name), body.stream()]
		.into_iter()
		.collect();
	tokens.push(TokenTree::Group(Group::new(Delimiter::Brace, instrumented_body)));

	Ok(TokenStream::from_iter(tokens))
}

// ::tracy_gizmos::zone!($text);
fn make_our_zone(name: &str) -> TokenStream {
	TokenStream::from_iter([
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("tracy_gizmos", Span::call_site())),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("zone", Span::call_site())),
		TokenTree::Punct(Punct::new('!', Spacing::Alone)),
		TokenTree::Group(
			Group::new(
				Delimiter::Brace,
				TokenStream::from_iter([
					TokenTree::Literal(Literal::string(name)),
				])
			)
		),
	])
}

struct Error {
	text:  &'static str,
	start: Span,
	end:   Span,
}

impl Error {
	fn new(text: &'static str, s: Span) -> Self {
		Self { text, start: s, end: s }
	}

	fn to_compile_error(&self) -> TokenStream {
		fn punct(c: char, s: Spacing, span: Span) -> TokenTree {
			TokenTree::Punct({
				let mut p = Punct::new(c, s);
				p.set_span(span);
				p
			})
		}

		TokenStream::from_iter([
			punct(':', Spacing::Joint, self.start),
			punct(':', Spacing::Alone, self.start),
			TokenTree::Ident(Ident::new("core", self.start)),
			punct(':', Spacing::Joint, self.start),
			punct(':', Spacing::Alone, self.start),
			TokenTree::Ident(Ident::new("compile_error", self.start)),
			punct('!', Spacing::Alone, self.start),
			TokenTree::Group({
				let mut g = Group::new(
					Delimiter::Brace,
					TokenStream::from_iter([
						TokenTree::Literal({
							let mut s = Literal::string(self.text);
							s.set_span(self.end);
							s
						})
					])
				);
				g.set_span(self.end);
				g
			}),
		])
	}
}
