use indexing::{
    scope,
    traits::{TrustedContainer, TrustedItem},
    Container, Range,
};

fn indices<Array, F, Out, T>(arr: Array, f: F) -> Out
where
    F: for<'id> FnOnce(Container<'id, Array>, Range<'id>) -> Out,
    Array: TrustedContainer<Item = T>,
    T: TrustedItem<Array>,
{
    scope(arr, move |v| {
        let range = v.range();
        f(v, range)
    })
}

fn main() {
    let arr1 = [1, 2, 3, 4, 5];
    let arr2 = [10, 20, 30];

    // do it twice to make NLL borrowck error orders deterministic

    indices(&arr1[..], |arr1, r1| {
        indices(&arr2[..], move |arr2, r2| {
            &arr2[r1]; //~ ERROR cannot infer an appropriate lifetime
            // &arr1[r2]; //~ ERROR cannot infer an appropriate lifetime
        });
    });

    indices(&arr1[..], |arr1, r1| {
        indices(&arr2[..], move |arr2, r2| {
            // &arr2[r1]; //~ ERROR cannot infer an appropriate lifetime
            &arr1[r2]; //~ ERROR cannot infer an appropriate lifetime
        });
    });
}
