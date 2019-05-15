use indexing::{scope, Range};

#[test]
fn join_add_proof() {
    let data = [1, 2, 3];
    scope(&data[..], move |v| {
        let r: Range = v.range();
        if let Some(r) = r.nonempty() {
            let (front, back) = r.frontiers();

            r.start();
            // test nonempty range methods
            front.join(r).unwrap().start();
            r.join(back).unwrap().start();
            front.join_cover(r).start();
            r.join_cover(back).start();
            r.join_cover(r).start();

            assert_eq!(front.join(r).unwrap(), r);
            assert_eq!(front.join_cover(back), r.erased());
            assert_eq!(back.join_cover(front), r.erased()); // DIFFERENCE FROM bluss/indexing
        }
    });
}

#[test]
fn range_split_nonempty() {
    let data = [1, 2, 3, 4, 5];
    scope(&data[..], move |v| {
        for i in 0..v.unit_len() {
            let r = v.vet_range(0..i).unwrap();
            if let Some(r) = r.nonempty() {
                let h = v.vet(i / 2).unwrap();
                let (a, b) = r.split_at(h).unwrap();
                assert!(b.len() > 0);
                assert_eq!(a.len() + b.len(), r.len());
                assert!(b.start().untrusted() < r.len());
            } else {
                let h = r.start(); // we can't vet a nonexistent halfway point
                let (a, b) = r.split_at(h).unwrap();
                assert_eq!(a.len(), 0);
                assert_eq!(b.len(), 0);
                assert_eq!(a.start(), b.start());
            }
        }
    });
}
