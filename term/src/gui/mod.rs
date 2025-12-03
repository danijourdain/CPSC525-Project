
pub mod account;
// pub mod desk;
pub mod formbutton;
pub mod ledger;
pub mod waiting;
pub mod login;

pub fn format_money(money: usize) -> String {
    let fragments = money
        .to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|f| f.into_iter().rev().collect::<String>())
        .rev()
        .collect::<Vec<_>>();

    let mut base = "$".to_string();

    for (i, frag) in fragments.iter().enumerate() {
        base += frag;
        if i != fragments.len() - 1 {
            base.push(',');
        }
    }

    base
}

pub fn format_money_accounting(money: isize) -> String {
    {
        if money < 0 {
            format!("({})", format_money((-money) as usize))
        } else if money == 0 {
            format!("{}", format_money(money as usize))
        } else {
            format!("{}", format_money(money as usize))
        }
    }
}
