use indexing::{scope, Range, proof::NonEmpty};

fn main() {
    let arr1 = [1, 2, 3, 4, 5];

    scope(&arr1[..], |v| {
        let r = v.range();
        if let Some(r) = r.nonempty() {
            let r: Range<'_, _, NonEmpty> = r;
            let (front, back) = r.frontiers();

            r.start();
            front.join(r).unwrap().start();
            r.join(back).unwrap().start();
            front.join_cover(r).start();
            r.join_cover(back).start();
            r.join_cover(r).start();

            // bluss uses `.last()`, which we don't have
            // instead we use `.observe_proof()`
            r.observe_proof();
            front.join(r).unwrap().observe_proof();
            r.join(back).unwrap().observe_proof();
            front.join_cover(r).observe_proof();
            r.join_cover(back).observe_proof();
            r.join_cover(r).observe_proof();

            front.join_cover(back).start();
            front.join_cover(back).observe_proof();
            //~^ ERROR no method named
        }
    });
}
