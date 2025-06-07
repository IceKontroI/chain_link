#[cfg(test)]
pub mod tests {
    
    use chain_link::*;

    /// Each link in a chain has input corresponding to the lower link's output.
    /// Cascade is: -1.5f32 -> -1i32 -> 4_294_967_295u32 -> "4,294,967,295".
    /// TODO it's suboptimal that we need to specify `Self::In<'_>` and `Self::Out<'_>`.
    ///      for some reason naming the types explicitly won't work here despite not having lifetimes.
    #[test]
    fn chain_link_cascade() {

        struct Pipeline;

        impl Chain<0> for Pipeline {
            type In<'a> = f32;
            type Out<'a> = i32;
        
            fn chain(input: Self::In<'_>) -> Self::Out<'_> {
                assert_eq!(input, -1.5);
                input as i32
            }
        }

        impl Chain<1> for Pipeline {
            type In<'a> = i32;
            type Out<'a> = u32;
        
            fn chain(input: Self::In<'_>) -> Self::Out<'_> {
                assert_eq!(input, -1);
                input as u32
            }
        }

        impl Chain<2> for Pipeline {
            type In<'a> = u32;
            type Out<'a> = String;

            fn chain(input: Self::In<'_>) -> Self::Out<'_> {
                assert_eq!(input, u32::MAX);
                let mut output = String::new();
                let mut n = input;
                while n >= 1_000 {
                    output = format!(",{:03}{}", n % 1_000, output);
                    n /= 1_000;
                }
                format!("{n}{}", output)
            }
        }

        impl Length for Pipeline {
            type Len = L<3>;
        }
        
        let input = -1.5;
        let actual = Pipeline::cascade(input);
        let expected = "4,294,967,295";
        assert_eq!(actual, expected);
    }

    #[test]
    fn portability_example() {

        // a 3rd party crate can define the following:
        pub struct NamesOf<T>(T);
        pub trait Name<const N: usize>
        where
            // doing this makes it impossible for the user to 
            // impl Length<Len = L<4>> then subsequently impl
            // Name<N> for any N >= 4, keeping it InRange
            Self: InRange<N, <Self as Length>::Len>
        {
            const NAME: &'static str;
        }
        impl<T: Length> Length for NamesOf<T> {
            type Len = T::Len;
        }
        impl<const N: usize, T: Name<N>> Chain<N> for NamesOf<T>
        where
            Self: InRange<N, Self::Len>
        {
            type In<'a> = String;
            type Out<'a> = String;
        
            fn chain(input: Self::In<'_>) -> Self::Out<'_> {
                format!("{}, {}", input, T::NAME)
            }
        }

        // the user implements Name<#> for their own custom struct
        // these must be implemented for all N in range of 0..Len,
        // or Cascade will not get implemented on NamesOf<Beatles>
        struct Beatles;
        impl Name<0> for Beatles {
            const NAME: &'static str = "John Lennon";
        }
        impl Name<1> for Beatles {
            const NAME: &'static str = "Paul McCartney";
        }
        impl Name<2> for Beatles {
            const NAME: &'static str = "George Harrison";
        }
        impl Name<3> for Beatles {
            const NAME: &'static str = "Ringo Starr";
        }
        
        // Length is user- implemented fpr Beatles struct
        // and auto-impls the length for NamesOf<Beatles>
        impl Length for Beatles {
            type Len = L<4>;
        }
        assert_eq!(Beatles::len(), 4);
        assert_eq!(NamesOf::<Beatles>::len(), 4);

        // fails to compile because Name<N> impls must be InRange of Length
        // impl Name<4> for Beatles {
        //     const NAME: &'static str = "George Martin";
        // }
        
        let header = "Names of the Beatles: ".to_owned();
        let result = NamesOf::<Beatles>::cascade(header);
        assert_eq!(
            result, 
            "Names of the Beatles: , John Lennon, Paul McCartney, George Harrison, Ringo Starr"
        );
    }
}
