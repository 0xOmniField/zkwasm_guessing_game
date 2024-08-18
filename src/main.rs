use std::io;
use std::cmp::Ordering;
use rand::Rng;


fn main() {
    println!("猜数字游戏开始!");

    let secret_number = rand::thread_rng().gen_range(1..101);
    // println!("The secret number is: {}", secret_number);
    loop {
        let mut guess = String::new();

        println!("请输入你想猜的数字.");
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        let guess: u32 = match guess.trim().parse() {
            Ok(num)=>num,
            Err(_)=> continue,
        };
        println!("您输入的数字为：{}", guess);
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("你猜的数字小了!"),
            Ordering::Greater => println!("你猜的数字大了!"),
            Ordering::Equal => {
                println!("你猜对了，游戏结束!");
                break;
            },
        }
    }

}
