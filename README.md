## Overview

Small crate to help you link different impls together and then chain their executions. I'm not aware of any crate or rust feature that lets you iterate on a sequence of different implementations of the same trait, so I made one. This was inspired by a project of mine that required trait impls on each field of a struct. I needed to be able to execute some logic on each qualifying named field of a struct, where each field had a unique impl of some shared trait. 

## 3rd party integration

For 3rd party crates that want to extend more specific functionality to its users can follow this pattern:
```
use chain_link::*;

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
```

Then your users can follow this pattern:
```
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
        
impl Length for Beatles {
    type Len = L<4>;
}
```

Which allows the following to be valid:
```
let header = "Names of the Beatles: ".to_owned();
let result = NamesOf::<Beatles>::cascade(header);
assert_eq!(
    result, 
    "Names of the Beatles: , John Lennon, Paul McCartney, George Harrison, Ringo Starr"
);
```

## Pipeline

For custom iteration where no specific trait impl is needed, and a simple pipeline-like cascade is required, this pattern is quite serviceable:
```
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
```

## Limitations & Future Features
This is very much a WIP crate, with some limitations I'd like to work on
* Due to limitations with rust's const generics, we're forced (I think) to set a hardcoded limit to supported valid lengths. Right now it's set at 16, and I noticed compiler slowdown at higher numbers. If rust supported constant math, I could use `{N - 1}` in the trait impl for `Chain<N>` and `Link<N>`. And if we could do conditional math like `{N < M}`, I could `impl InRange<N> for T: Length<Len = L<M>>`. This would remove the limitation, but I haven't found a way to do this in stable rust. This would also eliminate the need for `seq_macro` dependency, making it 100% native rust, which would be cool.
* Due to a known blindspot in rust's compiler, I'm forced to complicate the impl for `InRange<N, L<M>>`, making the business logic hard to follow. Not a big issue, but would be nice for a cleanup. Removing `L<M>` in general would be nice, but it's necessary for fixing this blindspot in other parts of the code too.
* It's possibly quite valuable to replace `L<M>` with `Num<M>` and use this type as a general-purpose compile-time constant indexing type for chains. However currently we're just working with numbered trait impls, so it's not clear exactly what type a user might want to receive as output to the index function.
* A lot of logic revolves around requiring the `Length` trait. This produces a lot of boilerplate code, but also serves as a guardrail preventing users from implementing Chain<N> where N >= the Length. This helps prevent a scenario where a user implements a chain of some initial length, then adds another link but forgets to increment the Length, resulting in no change in Cascade behavior.
* Cascade can only ever execute for a chain of 0..Length, which is suitable for most common needs, but it would be a lot better if we could trigger it for any arbitrary valid range. Not sure of concrete use-cases but the API should be flexible enough to handle this use-case.
* Portability is workable, but has limitations because 3rd party crates cannot impl `Chain<N>` for `T: CustomTrait<N>` because `Chain` is defined in an external crate and `CustomTrait` is defined locally. It's still possible to do, but adds a lot of boilerplate code which is suboptimal. See the Beatles example above for reference.
* Structs are currently restricted to one `Chain` impl. Workable patterns exist, similar to the Beatles example above. Without adding a second generic param to `Chain`, we can only have one `Chain` impl, but it keeps business logic clean, so this is acceptable.
* Often we don't care to define custom In/Out types per `Chain` impl, as is the case for uniform `Cascades`. A new `Sequence` trait could make usability much simpler, keeping In/Out the same. Also with uniform In/Out comes some increase in flexibility; things like the ability to iterate in reverse, or execute only a select few indexes of the sequence.