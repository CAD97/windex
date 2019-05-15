use indexing::{scope, Index, Range};

#[test]
fn nonempty() {
    let data = [0, 1, 2, 3, 4, 5];
    scope(&data[..], |data| {
        let mut r = data.range::<u32>().nonempty().unwrap();
        assert_eq!(data[r.start()], 0);

        assert!(r.advance_in(&data));
        assert_eq!(data[r.start()], 1);

        assert!(r.advance_in(&data));
        assert_eq!(data[r.start()], 2);

        // skip to end
        while r.advance_in(&data) {}
        assert_eq!(data[r.start()], 5);
    });
}

#[test]
fn contains() {
    let data = [0, 1, 2, 3, 4, 5];
    scope(&data[..], |data| {
        let r = data.range::<u32>();
        for i in 0..data.unit_len() {
            assert!(r.contains_in(i, &data).is_som());
            assert_eq!(r.contains_in(i, &data).unwrap(), data.vet(i).unwrap());
        }
        assert!(r.contains_in(r.len(), &data).is_none());
        assert!(data.vet(r.len()).is_err());
    })
}

#[test]
fn is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() { }
    assert_send_sync::<Index<'_>>();
    assert_send_sync::<Range<'_>>();
}
