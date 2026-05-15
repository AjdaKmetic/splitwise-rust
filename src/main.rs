use settlemate_rust::{
    models::expense::Expense,
    models::group::Group,
    services::split::Split,
    models::user::{User},
};

fn main() {
    let janez = User::new(1, "Janez Novak", "janeznovak@example.com");
    let marija = User::new(2, "Marija Novak", "marijanovak@example.com");
    let mut group = Group::new(1, "Amsterdam");
    group.add_member(janez.id);
    group.add_member(marija.id);

    let expenses = vec![
        Expense::new(
            1,
            "Hotel".into(),
            200.0,
            janez.id,
            Some(group.id),
            Split::Equal(vec![janez.id, marija.id]),
        ),
        Expense::new(
            2,
            "Vecerja".into(),
            100.0,
            marija.id,
            Some(group.id),
            Split::Exact(vec![(janez.id, 70.0), (marija.id, 30.0)]),
        ),
    ];

    let balances = settlemate_rust::services::balance::Balance::calculate_balances(&expenses);
    println!("Balances: {:?}", balances);

    let transactions = settlemate_rust::services::simplify::simplify_debts(&balances);
    println!("Simplified Transactions: {:?}", transactions);
}
