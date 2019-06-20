use windex::scope_val;

fn main() {
    let v = vec![0];
    scope_val(v, |mut v| {
        let _ix = v.vet(0).unwrap();
        let r = v.as_ref_mut().into_untrusted();
        r.clear();
        // ix is now invalid logically but not statically
    })
}

// Remember to update the actual doc when this test changes!
