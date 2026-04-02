fn main() {
    println!("Hello, world!");

    // 1. Ownership
    // stack

    // let x = 5;
    // let y = x;

    // println!("The value of x is {}", x);
    // println!("The value of y is {}", y);

    // // Heap
    // let s1 = String::from("vipin");

    // let s2 = s1;

    // //? println!("{}", s1); // Error:  borrow of moved value: `s1` value borrowed here after move

    // println!("{}", s2);

    // // Using Clone

    // let s3 = s2.clone(); // Creating copy on heap

    // println!("{}", s2);
    // println!("{}", s3);

    // 2. Borrowing

    // let s1 = String::from("hello");
    // // let len = get_length(s1);

    // // println!("s1 {s1} and len {len}",); //? Error:  borrow of moved value: `s1` value borrowed here after move

    // let (s1, len) = get_len_with_brow(s1);
    // println!("s1 {s1} and len {len}");

    // 3. Reference

    let s1 = String::from("hello");
    let len = get_with_ref(&s1);
    println!("s1 {s1} and len {len}");

    let mut s1 = String::from("hello");
    update_str(&mut s1);
    println!("{}", s1);
}

fn get_length(s: String) -> usize {
    s.len()
}

fn get_len_with_brow(s: String) -> (String, usize) {
    let len = s.len();
    (s, len)
}

fn get_with_ref(s: &String) -> usize {
    s.len()
}

fn update_str(s: &mut String) {
    s.push_str(" world");
}
