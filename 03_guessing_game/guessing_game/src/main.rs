use std::cmp::Ordering;
use std::io;

fn main() {
    println!("Guess Game!");
    println!("Please input your guess.");

    let secret_number = rand::random_range(1..=100);
    // println!("The secret number is: {}", secret_number);

    loop {
        let mut guess: String = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to get user input");

        println!("You guessed: {}", guess);

        let guess_number: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please type a number!");
                continue;
            }
        };

        match guess_number.cmp(&secret_number) {
            Ordering::Greater => println!("Too big!"),
            Ordering::Less => println!("Too small!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        };
    }
}
