use ragu_macros::repr256;

#[test]
fn correctness() {
    assert_eq!(
        repr256!(0x1745569a0a3e30142186c3038ea05e697e3b83af4a4ba3ba79c47c573ac410f7),
        [
            8774274688114495735,
            9096008661293835194,
            2415832670176042601,
            1676841655862177812
        ]
    );
    assert_eq!(repr256!(0x1), [1, 0, 0, 0]);
    assert_eq!(
        repr256!(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff),
        [
            0xffffffffffffffff,
            0xffffffffffffffff,
            0xffffffffffffffff,
            0xffffffffffffffff
        ]
    );
}
