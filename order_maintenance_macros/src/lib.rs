use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Bracket,
    Attribute, LitFloat, LitInt, Token, Visibility,
};

/// Declaration for threshold range.
///
/// Looks like this:
///
/// ```no_compile
/// {vis?} const {name}: [[{begin}..={end}; {bits}]; {count}];
/// ```
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

        // TODO: warn if bits is not 32/64/a reasonable value?

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
/// # use order_maintenance_macros::*;
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

/// Compute the capacities for a particular threshold.
fn capacities_for_threshold(t: f64, bits: usize) -> Vec<usize> {
    // TODO: assert that `t` is between 1.0 and 2.0?
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
            629979908206816,
            1145418014921484,
            2082578208948154,
            3786505834451189,
            6884556062638524,
            12517374659342768,
            22758863016986852,
        ];

        assert_eq!(t1_1.to_vec(), capacities_for_threshold(1.1, 64))
    }

    #[test]
    fn test_t1_2() {
        let t1_2: [usize; 64] = [
            1,
            1,
            2,
            4,
            7,
            12,
            21,
            35,
            59,
            99,
            165,
            275,
            459,
            765,
            1276,
            2126,
            3544,
            5907,
            9846,
            16410,
            27351,
            45585,
            75975,
            126625,
            211042,
            351737,
            586229,
            977048,
            1628414,
            2714024,
            4523373,
            7538956,
            12564927,
            20941545,
            34902576,
            58170960,
            96951601,
            161586002,
            269310003,
            448850005,
            748083342,
            1246805571,
            2078009285,
            3463348809,
            5772248015,
            9620413359,
            16034022265,
            26723370443,
            44538950738,
            74231584564,
            123719307607,
            206198846012,
            343664743354,
            572774572256,
            954624287094,
            1591040478490,
            2651734130817,
            4419556884696,
            7365928141161,
            12276546901935,
            20460911503225,
            34101519172041,
            56835865286736,
            94726442144560,
        ];
        assert_eq!(t1_2.to_vec(), capacities_for_threshold(1.2, 64))
    }

    #[test]
    fn check_t1_25() {
        let t1_25: [usize; 64] = [
            1,
            1,
            2,
            4,
            6,
            10,
            16,
            26,
            42,
            68,
            109,
            175,
            281,
            450,
            720,
            1152,
            1844,
            2951,
            4722,
            7555,
            12089,
            19342,
            30948,
            49517,
            79228,
            126765,
            202824,
            324518,
            519229,
            830767,
            1329227,
            2126764,
            3402823,
            5444517,
            8711228,
            13937965,
            22300745,
            35681192,
            57089907,
            91343852,
            146150163,
            233840261,
            374144419,
            598631070,
            957809713,
            1532495540,
            2451992865,
            3923188584,
            6277101735,
            10043362776,
            16069380442,
            25711008708,
            41137613933,
            65820182292,
            105312291668,
            168499666669,
            269599466671,
            431359146674,
            690174634679,
            1104279415486,
            1766847064778,
            2826955303645,
            4523128485832,
            7237005577332,
        ];
        assert_eq!(t1_25.to_vec(), capacities_for_threshold(1.25, 64))
    }

    #[test]
    fn check_t1_4() {
        let t1_4: [usize; 64] = [
            1, 1, 2, 2, 4, 5, 8, 12, 17, 24, 35, 50, 72, 103, 147, 210, 300, 429, 614, 877, 1253,
            1790, 2557, 3653, 5219, 7456, 10652, 15217, 21739, 31056, 44366, 63381, 90544, 129349,
            184784, 263978, 377112, 538731, 769616, 1099452, 1570646, 2243780, 3205400, 4579143,
            6541633, 9345191, 13350273, 19071819, 27245455, 38922079, 55602971, 79432816,
            113475451, 162107787, 231582554, 330832220, 472617457, 675167795, 964525422,
            1377893461, 1968419230, 2812027471, 4017182102, 5738831574,
        ];
        assert_eq!(t1_4.to_vec(), capacities_for_threshold(1.4, 64))
    }

    #[test]
    fn check_t1_8() {
        let t1_8: [usize; 64] = [
            1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15,
            17, 19, 21, 23, 26, 29, 32, 35, 39, 44, 49, 54, 60, 67, 75, 83, 92, 103, 114, 127, 141,
            157, 174, 194, 215, 239, 266, 295, 328, 365, 405, 450, 500, 556, 618, 687, 763,
        ];
        assert_eq!(t1_8.to_vec(), capacities_for_threshold(1.8, 64))
    }

    #[test]
    fn check_t1_85() {
        let t1_85: [usize; 64] = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8,
            9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 22, 24, 26, 28, 30, 33, 36, 39, 42, 45, 49,
            53, 57, 62, 67, 72, 78, 85, 91, 99, 107, 116, 125, 135,
        ];
        assert_eq!(t1_85.to_vec(), capacities_for_threshold(1.85, 64))
    }
}
