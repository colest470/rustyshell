use std::io;

enum Direction {
    North,
    East,
    West,
    South,
}

fn test_direction() {
    let direction = Direction::North;
    
    println!("Direction is: {:?}", direction);
}