pub fn is_flag_true(data: u8, flag: u8) -> bool {
    match data & flag {
        0 => false,
        _ => true,
    }
}
