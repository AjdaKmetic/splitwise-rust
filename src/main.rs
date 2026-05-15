use sea_orm::{EntityTrait};

use settlemate_rust::{
    // models::expense::Expense,
    // models::group::Group,
    // services::split::Split,
    // models::user::{User},
    database::connect,
    entities::users,
    services::user_service::create_user,
};

#[tokio::main]
async fn main() {

    /*
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

    let balances =
        settlemate_rust::services::balance::Balance::calculate_balances(&expenses);

    println!("Balances: {:?}", balances);

    let transactions =
        settlemate_rust::services::simplify::simplify_debts(&balances);

    println!("Simplified Transactions: {:?}", transactions);
    */

    let db = connect()
        .await
        .expect("Povezava z bazo ni uspela");

    println!("Povezava z bazo deluje.");

/*    let new_user = users::ActiveModel {
        name: Set("Janez Novak".to_string()),
        email: Set("janez@example.com".to_string()),
        ..Default::default()
    };

    let result = new_user
        .insert(&db)
        .await
        .expect("Dodajanje uporabnika ni uspelo");

    println!("Dodan uporabnik: {:?}", result);
*/

    let result = create_user(
        &db,
        "Janez Novak",
        "janez@example.com",
    )
    .await
    .expect("Dodajanje uporabnika ni uspelo");

    let all_users = users::Entity::find()
        .all(&db)
        .await
        .expect("Branje uporabnikov ni uspelo");

    println!("Vsi uporabniki v bazi:");
    for user in all_users {
        println!("{} - {} ({})", user.id, user.name, user.email);
    }

}