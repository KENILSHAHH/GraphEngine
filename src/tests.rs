use super::*;
use std::collections::HashMap;

#[test]
fn test_polynomial() {
    let mut builder = Builder::new();
    let x = builder.init();
    let x_squared = builder.mul(&x, &x);
    let five = builder.constant(3);
    let x_squared_plus_x = builder.add(&x_squared, &x);
    let _y = builder.add(&x_squared_plus_x, &five);

    let mut inputs = HashMap::new();
    inputs.insert(x.id, 3);
    builder.fill_nodes(inputs);
    assert!(builder.check_constraints());
}

#[test]
fn test_hint_division() {
    let mut builder = Builder::new();
    let a = builder.init();
    let one = builder.constant(1);
    let b = builder.add(&a, &one);
    let c = builder.hint(vec![b.clone()], |vals| vals[0] / 8);
    let eight = builder.constant(8);
    let c_times_8 = builder.mul(&c, &eight);
    builder.assert_equal(&b, &c_times_8);

    let mut inputs = HashMap::new();
    inputs.insert(a.id, 7);
    builder.fill_nodes(inputs);
    assert!(builder.check_constraints());
}

#[test]
fn test_hint_sqrt() {
    let mut builder = Builder::new();
    let x = builder.init();
    let seven = builder.constant(7);
    let x_plus_seven = builder.add(&x, &seven);
    let sqrt_node = builder.hint(vec![x_plus_seven.clone()], |vals| {
        (vals[0] as f64).sqrt() as u32
    });
    let computed_sq = builder.mul(&sqrt_node, &sqrt_node);
    builder.assert_equal(&computed_sq, &x_plus_seven);

    let mut inputs = HashMap::new();
    inputs.insert(x.id, 2);
    builder.fill_nodes(inputs);
    assert!(builder.check_constraints());
}
