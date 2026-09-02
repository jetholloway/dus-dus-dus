fn main() {
    let result = [1, 5, 3, 2, 4].iter().max_by_key(|&x| {
        println!("{x} => {}", x * x);
        x * x
    });
    println!("{result:?}");
}
