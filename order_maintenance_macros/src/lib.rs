use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Bracket,
    Attribute, LitFloat, LitInt, Token, Visibility,
};

struct ThresholdRange {
    attrs: Vec<Attribute>,
    vis: Visibility,
    _const: Token![const],
    name: Ident,
    _colon: Token![:],
    _bracket1: Bracket,
    _bracket2: Bracket,
    begin: LitFloat,
    _dotdoteq: Token![..=],
    end: LitFloat,
    _semi2: Token![;],
    bits: LitInt,
    _semi1: Token![;],
    count: LitInt,
    _semi: Token![;],
}

impl Parse for ThresholdRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content1;
        let content2;

        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            _const: input.parse()?,
            name: input.parse()?,
            _colon: input.parse()?,
            _bracket1: bracketed!(content1 in input),
            _bracket2: bracketed!(content2 in content1),
            begin: content2.parse()?,
            _dotdoteq: content2.parse()?,
            end: content2.parse()?,
            _semi2: content2.parse()?,
            bits: content2.parse()?,
            _semi1: content1.parse()?,
            count: content1.parse()?,
            _semi: input.parse()?,
        })
    }
}

impl ThresholdRange {
    fn generate(&self) -> syn::Result<TokenStream> {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let name = &self.name;

        let begin: f64 = self.begin.base10_parse()?;
        let end: f64 = self.end.base10_parse()?;
        let bits: usize = self.bits.base10_parse()?;
        let count: usize = self.count.base10_parse()?;

        let gap = (end - begin) / (count as f64);

        let capas: Vec<Vec<usize>> = (0..count)
            .map(|i| capacities_for_threshold(begin + (i as f64) * gap, bits))
            .collect();

        Ok(quote! {
            #( #attrs )*
            #vis const #name: [[usize; #bits]; #count] = [#( [#(#capas),*] ),*];
        })
    }
}

/// Generate the capacities for a range of thresholds.
///
/// Example:
///
/// ```
/// generate_capacities! {
///     /// Capacities for 17 thresholds in the range `(1.1..=1.9)` (inclusive) with 64-bit tags.
///     const CAPAS: [[1.1..=1.9; 64]; 17];
/// }
/// ```
///
#[proc_macro]
pub fn generate_capacities(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as ThresholdRange)
        .generate()
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn capacities_for_threshold(t: f64, bits: usize) -> Vec<usize> {
    (0..bits)
        .map(|b| ((2.0f64 / t).powi(b as i32).floor()) as usize)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_t1_1() {
        let t1_1: [usize; 64] = [
            1,
            1,
            3,
            6,
            10,
            19,
            36,
            65,
            119,
            217,
            394,
            717,
            1305,
            2372,
            4314,
            7844,
            14262,
            25931,
            47148,
            85725,
            155864,
            283389,
            515253,
            936824,
            1703316,
            3096939,
            5630799,
            10237817,
            18614213,
            33844024,
            61534590,
            111881073,
            203420134,
            369854789,
            672463253,
            1222660460,
            2223019018,
            4041852760,
            7348823201,
            13361496730,
            24293630418,
            44170237123,
            80309522043,
            146017312806,
            265486023284,
            482701860517,
            877639746395,
            1595708629810,
            2901288417837,
            5275069850613,
            9591036092023,
            17438247440043,
            31705904436442,
            57647098975350,
            104812907227909,
            190568922232562,
            346488949513749,
            629979908206817,
            1145418014921486,
            2082578208948156,
            3786505834451193,
            6884556062638532,
            12517374659342786,
            22758863016986884,
        ];

        assert_eq!(t1_1.to_vec(), capacities_for_threshold(1.1, 64))
    }

    #[test]
    fn check_t1_8() {
        let t1_8: [usize; 64] = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8,
            9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 22, 24, 26, 28, 30, 33, 36, 39, 42, 45, 49,
            53, 57, 62, 67, 72, 78, 85, 91, 99, 107, 116, 125, 135,
        ];
        assert_eq!(t1_8.to_vec(), capacities_for_threshold(1.8, 64))
    }
}
