use sealed::Len;
use seq_macro::seq;

/// WIP I'm stuck between requiring `Length` trait and eliminating it
///     If it's kept, it ensures the user cannot implement Chain past its length
///         Which is a good guardrail, since it ensures the user always knows the chain length
///     However it makes the API have more boilerplate
///     But most importantly, it prevents us from cascading at custom index ranges
///         And that's a feature that should be added in the future
/// 
/// WIP This is technically portable but requires some boilerplate on a newtype from the user
///
/// WIP currently structs are restricted to one type of chain impl
///      so you can't end up with mulitple types of cascades on the same type
///      this is desireable to reduce verbosity (which there is already too much of IMO)
///      this can be simplified by supporting sequences:
/// 
/// WIP often we don't want to be able to transform inputs -> outputs during the chain
///     it would be helpful for the user to be able to implement a Sequence<N> wrapper
///     this would wrap the Chain<N> implementation, forcing its `In` and `Out` to be the same value
const _WIP: () = ();

/// Require all Length::Len types to be `L<const N: usize>` so that the InRange traits can be
/// implemented with the same marker type. This is a workaround to the rust compiler blindspot
/// which doesn't recognize certain non-overlapping trait impls and throws a compiler error.
/// 
/// Fails:    `impl<T: Length<Len = L<N>>> InRange<I>         for T {}`
/// Fails:    `impl<T: Length>             InRange<I, T::Len> for T {}`
/// Succeeds: `impl<T: Length<Len = L<N>>> InRange<I, L<N>>   for T {}`
/// 
/// Even though there's no way anything can impl Len more than once.
mod sealed {
    pub trait Len {
        const LEN: usize;
    }
}

// TODO I really hate this L<N> requirement, but we seem to need it to get around rust's
//      compiler bug where non-overlapping trait impls are detected as overlapping, when
//      using associated type equality as an impl condition
// IDEA but maybe we can use it as the index into a compile-time indexing library?
pub struct L<const N: usize>;
impl<const N: usize> Len for L<N> {
    const LEN: usize = N;
}

pub trait Length {
    type Len: sealed::Len;

    fn len() -> usize {
        <Self::Len as Len>::LEN
    }
}

pub trait InRange<const N: usize, L>: Length {}

pub trait Chain<const N: usize>
where
    Self: InRange<N, <Self as Length>::Len>,
{
    type In<'a>;
    type Out<'a>;

    fn chain(input: Self::In<'_>) -> Self::Out<'_>;
}

pub trait Link<const N: usize> {
    type In<'a>;
    type Out<'a>;

    fn link<'a>(input: Self::In<'a>) -> Self::Out<'a>;
}

impl<T: Chain<0>> Link<1> for T {
    type In<'a> = <T as Chain<0>>::In<'a>;
    type Out<'a> = <T as Chain<0>>::Out<'a>;

    fn link(input: Self::In<'_>) -> Self::Out<'_> {
        return <T as Chain<0>>::chain(input);
    } 
}

// TODO currently an annoying limitation is the hardcoded limit to how many things can be chained
//      realistically it's not such a problem, since nobody's gonna implement more than 32 chains manually
//      but if they do it via macro, it could become a limitation, but this is the best way I found so far

// add in-range marker trait impls for anything with up to length 32 (0..31 addressable)
seq!(N in 1..=32 {
    seq!(I in 0..N {
        impl<T: Length<Len = L<N>>> InRange<I, L<N>> for T {}
    });
});

// type gymnastics so that the input of the next link is the output of the previous one
seq!(N in 2..=32 {
    impl<T> Link<N> for T
    where
        T: Chain<0>,
        for<'a> T: Link<{N - 1}, In<'a> = <T as Chain<0>>::In<'a>>,
        for<'a> T: Chain<{N - 1}, In<'a> = <T as Link<{N - 1}>>::Out<'a>>,
    {
        type In<'a> = <T as Chain<0>>::In<'a>;
        type Out<'a> = <T as Chain<{N - 1}>>::Out<'a>;

        fn link(input: Self::In<'_>) -> Self::Out<'_> {
            let out = <T as Link<{N - 1}>>::link(input);
            return <T as Chain<{N - 1}>>::chain(out);
        }
    }
});

// support chaining from 0..N for any T: Length<Len = L<N>> that has a Link at `N - 1`

pub trait Cascade {
    type In<'a>;
    type Out<'a>;

    fn cascade(input: Self::In<'_>) -> Self::Out<'_>;
}

impl<const N: usize, T: Link<N> + Length<Len = L<N>>> Cascade for T {
    type In<'a> = T::In<'a>;
    type Out<'a> = T::Out<'a>;

    fn cascade(input: Self::In<'_>) -> Self::Out<'_> {
        return <T as Link::<N>>::link(input);
    }
}
