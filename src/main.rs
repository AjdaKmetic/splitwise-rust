use settlemate_rust::{
    models::expense::Expense,
    models::group::Group,
    services::split::Split,
    models::user::{User, UserId},
};

fn main() {
    let Janez = User::new(1, "Janez Novak", "janeznovak@example.com");
    let Marija = User::new(2, "Marija Novak", "marijanovak@example.com");
    let mut group = Group::new(1, "Amsterdam");
    group.add_member(Janez.id);
    group.add_member(Marija.id);

    let expenses = vec![
        Expense::new(
            1,
            "Hotel".into(),
            200.0,
            Janez.id,
            Some(group.id),
            Split::Equal(vec![Janez.id, Marija.id]),
        ),
        Expense::new(
            2,
            "Dinner".into(),
            100.0,
            Marija.id,
            Some(group.id),
            Split::Exact(vec![(Janez.id, 70.0), (Marija.id, 30.0)]),
        ),
    ];

}
