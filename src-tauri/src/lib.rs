use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use settlemate_rust::{
    models::{
        expense::Expense,
        group::Group,
        payment::Payment,
        user::User,
    },
    services::{
        balance::Balance,
        simplify::simplify_debts,
        split::Split,
    },
};

#[derive(Default)]
struct AppData {
    users: Vec<User>,
    groups: Vec<Group>,
    expenses: Vec<Expense>,
    payments: Vec<Payment>,
    next_user_id: u64,
    next_group_id: u64,
    next_expense_id: u64,
    next_payment_id: u64,
    current_user_id: Option<u64>,
}

struct AppState(Mutex<AppData>);

#[derive(Serialize, Clone)]
struct FriendDto { id: u64, name: String, balance: f64 }

#[derive(Serialize, Clone)]
struct GroupDto {
    id: u64,
    name: String,
    member_ids: Vec<u64>,
    members: Vec<String>,
    expense_count: usize,
    has_outstanding: bool,
    my_balance: f64,
}

#[derive(Serialize)]
struct ExpenseDto {
    id: u64,
    description: String,
    amount: f64,
    paid_by: String,
    paid_by_id: u64,
    participants: Vec<String>,
    participant_ids: Vec<u64>,
    group_id: Option<u64>,
    group_name: Option<String>,
    created_at: u64,
}

#[derive(Serialize, Clone)]
struct PaymentDto {
    id: u64,
    from_id: u64,
    from_name: String,
    to_id: u64,
    to_name: String,
    amount: f64,
    group_id: Option<u64>,
    group_name: Option<String>,
    created_at: u64,
}

#[derive(Serialize, Clone)]
struct BalanceDto {
    user_id: u64,
    name: String,
    amount: f64,
}

#[derive(Serialize)]
struct SettlementDto { from: String, to: String, amount: f64 }

// HELPERS 

fn name_of(data: &AppData, id: u64) -> String {
    data.users.iter()
        .find(|u| u.id == id)
        .map(|u| u.name().to_string())
        .unwrap_or_else(|| format!("?{id}"))
}

fn group_name_of(data: &AppData, id: u64) -> Option<String> {
    data.groups.iter().find(|g| g.id == id).map(|g| g.name().to_string())
}

fn expense_to_dto(e: &Expense, data: &AppData) -> ExpenseDto {
    let gid = e.group_id();
    let (participants, participant_ids): (Vec<String>, Vec<u64>) = match e.splits() {
        Split::Equal(ids) => ids.iter().map(|&id| (name_of(data, id), id)).unzip(),
        Split::Exact(pairs) => pairs.iter().map(|(id, _)| (name_of(data, *id), *id)).unzip(),
    };
    ExpenseDto {
        id: e.id,
        description: e.description().to_string(),
        amount: e.amount(),
        paid_by: name_of(data, e.paid_by()),
        paid_by_id: e.paid_by(),
        participants,
        participant_ids,
        group_id: gid,
        group_name: gid.and_then(|id| group_name_of(data, id)),
        created_at: e.created_at(),
    }
}

fn payment_to_dto(p: &Payment, data: &AppData) -> PaymentDto {
    PaymentDto {
        id: p.id,
        from_id: p.from_id(),
        from_name: name_of(data, p.from_id()),
        to_id: p.to_id(),
        to_name: name_of(data, p.to_id()),
        amount: p.amount(),
        group_id: p.group_id(),
        group_name: p.group_id().and_then(|id| group_name_of(data, id)),
        created_at: p.created_at(),
    }
}

// FRIENDS 

#[tauri::command]
fn add_friend(name: String, state: State<AppState>) -> FriendDto {
    let mut data = state.0.lock().unwrap();
    data.next_user_id += 1;
    let id = data.next_user_id;
    data.users.push(User::new(id, &name, ""));
    FriendDto { id, name, balance: 0.0 }
}

#[tauri::command]
fn list_friends(state: State<AppState>) -> Vec<FriendDto> {
    let data = state.0.lock().unwrap();
    match data.current_user_id {
        Some(my_id) => {
            let pairwise = Balance::pairwise_balances(&data.expenses, &data.payments, my_id);
            data.users.iter()
                .filter(|u| u.id != my_id)
                .map(|u| FriendDto {
                    id: u.id,
                    name: u.name().to_string(),
                    balance: pairwise.get(&u.id).copied().unwrap_or(0.0),
                })
                .collect()
        }
        None => {
            let balances = Balance::balances_with_payments(&data.expenses, &data.payments);
            data.users.iter()
                .map(|u| FriendDto {
                    id: u.id,
                    name: u.name().to_string(),
                    balance: balances.get(&u.id).copied().unwrap_or(0.0),
                })
                .collect()
        }
    }
}

#[tauri::command]
fn list_expenses_for_friend(friend_id: u64, state: State<AppState>) -> Vec<ExpenseDto> {
    let data = state.0.lock().unwrap();
    data.expenses.iter().rev()
        .filter(|e| {
            if e.paid_by() == friend_id { return true; }
            match e.splits() {
                Split::Equal(ids) => ids.contains(&friend_id),
                Split::Exact(pairs) => pairs.iter().any(|(id, _)| *id == friend_id),
            }
        })
        .map(|e| expense_to_dto(e, &data))
        .collect()
}

#[tauri::command]
fn friend_breakdown(friend_id: u64, state: State<AppState>) -> Vec<BalanceDto> {
    let data = state.0.lock().unwrap();
    let breakdown = Balance::pairwise_balances(&data.expenses, &data.payments, friend_id);
    breakdown.iter()
        .filter(|(_, amt)| amt.abs() > 0.005)
        .map(|(&id, &amount)| BalanceDto { user_id: id, name: name_of(&data, id), amount })
        .collect()
}

// GROUPS 

#[tauri::command]
fn create_group(name: String, member_ids: Vec<u64>, state: State<AppState>) -> Result<GroupDto, String> {
    if name.trim().is_empty() { return Err("Group name is required.".into()); }
    if member_ids.is_empty() { return Err("Select at least one member.".into()); }
    let mut data = state.0.lock().unwrap();
    data.next_group_id += 1;
    let id = data.next_group_id;
    let mut group = Group::new(id, &name);
    for &mid in &member_ids {
        group.add_member(mid);
    }
    data.groups.push(group);
    let members: Vec<String> = member_ids.iter().map(|id| name_of(&data, *id)).collect();
    Ok(GroupDto { id, name, member_ids, members, expense_count: 0, has_outstanding: false, my_balance: 0.0 })
}

#[tauri::command]
fn list_groups(state: State<AppState>) -> Vec<GroupDto> {
    let data = state.0.lock().unwrap();
    data.groups.iter().map(|g| {
        let group_expenses: Vec<Expense> = data.expenses.iter()
            .filter(|e| e.group_id() == Some(g.id))
            .cloned()
            .collect();
        let group_payments: Vec<Payment> = data.payments.iter()
            .filter(|p| p.group_id() == Some(g.id))
            .cloned()
            .collect();
        let balances = Balance::balances_with_payments(&group_expenses, &group_payments);
        let has_outstanding = balances.values().any(|&v| v.abs() > 0.01);
        let my_balance = match data.current_user_id {
            Some(my_id) => balances.get(&my_id).copied().unwrap_or(0.0),
            None => 0.0,
        };
        let member_ids: Vec<u64> = g.members().to_vec();
        let members = member_ids.iter().map(|&id| name_of(&data, id)).collect();
        GroupDto {
            id: g.id,
            name: g.name().to_string(),
            member_ids,
            members,
            expense_count: group_expenses.len(),
            has_outstanding,
            my_balance,
        }
    }).collect()
}

#[tauri::command]
fn group_balances(group_id: u64, state: State<AppState>) -> Vec<BalanceDto> {
    let data = state.0.lock().unwrap();
    let group_expenses: Vec<Expense> = data.expenses.iter()
        .filter(|e| e.group_id() == Some(group_id))
        .cloned()
        .collect();
    let group_payments: Vec<Payment> = data.payments.iter()
        .filter(|p| p.group_id() == Some(group_id))
        .cloned()
        .collect();
    let balances = Balance::balances_with_payments(&group_expenses, &group_payments);
    let member_ids: Vec<u64> = data.groups.iter()
        .find(|g| g.id == group_id)
        .map(|g| g.members().to_vec())
        .unwrap_or_default();
    let mut result: Vec<BalanceDto> = member_ids.iter()
        .map(|&id| BalanceDto {
            user_id: id,
            name: name_of(&data, id),
            amount: balances.get(&id).copied().unwrap_or(0.0),
        })
        .collect();
    result.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal));
    result
}

#[tauri::command]
fn simplify_group(group_id: u64, state: State<AppState>) -> Vec<SettlementDto> {
    let data = state.0.lock().unwrap();
    let group_expenses: Vec<Expense> = data.expenses.iter()
        .filter(|e| e.group_id() == Some(group_id))
        .cloned()
        .collect();
    let group_payments: Vec<Payment> = data.payments.iter()
        .filter(|p| p.group_id() == Some(group_id))
        .cloned()
        .collect();
    let balances = Balance::balances_with_payments(&group_expenses, &group_payments);
    simplify_debts(&balances).iter().map(|d| SettlementDto {
        from: name_of(&data, d.from()),
        to: name_of(&data, d.to()),
        amount: d.amount(),
    }).collect()
}

#[tauri::command]
fn list_expenses_for_group(group_id: u64, state: State<AppState>) -> Vec<ExpenseDto> {
    let data = state.0.lock().unwrap();
    data.expenses.iter()
        .filter(|e| e.group_id() == Some(group_id))
        .rev()
        .map(|e| expense_to_dto(e, &data))
        .collect()
}

#[tauri::command]
fn delete_group(group_id: u64, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    if !data.groups.iter().any(|g| g.id == group_id) {
        return Err("Group not found".to_string());
    }
    data.groups.retain(|g| g.id != group_id);
    data.expenses.retain(|e| e.group_id() != Some(group_id));
    data.payments.retain(|p| p.group_id() != Some(group_id));
    Ok(())
}

#[tauri::command]
fn add_members_to_group(group_id: u64, member_ids: Vec<u64>, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    let group = data.groups.iter_mut().find(|g| g.id == group_id)
        .ok_or_else(|| "Group not found".to_string())?;
    for id in member_ids {
        group.add_member(id);
    }
    Ok(())
}

#[tauri::command]
fn remove_member_from_group(group_id: u64, user_id: u64, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    let group = data.groups.iter_mut().find(|g| g.id == group_id)
        .ok_or_else(|| "Group not found".to_string())?;
    if group.member_count() <= 1 {
        return Err("Cannot remove the last member of a group".to_string());
    }
    group.remove_member(user_id);
    Ok(())
}

// EXPENSES 

#[tauri::command]
fn add_expense(
    description: String,
    amount: f64,
    paid_by: u64,
    participants: Vec<u64>,
    group_id: Option<u64>,
    state: State<AppState>,
) -> Result<(), String> {
    if amount <= 0.0 { return Err("Amount must be positive.".into()); }
    if participants.is_empty() { return Err("Select at least one participant.".into()); }
    let mut data = state.0.lock().unwrap();
    data.next_expense_id += 1;
    let id = data.next_expense_id;
    let split = Split::new_equal(participants).map_err(|e| e)?;
    data.expenses.push(Expense::new(id, &description, amount, paid_by, group_id, split));
    Ok(())
}

#[tauri::command]
fn update_expense(
    id: u64,
    description: String,
    amount: f64,
    paid_by: u64,
    participants: Vec<u64>,
    group_id: Option<u64>,
    state: State<AppState>,
) -> Result<(), String> {
    if amount <= 0.0 { return Err("Amount must be positive.".into()); }
    if participants.is_empty() { return Err("Select at least one participant.".into()); }
    let mut data = state.0.lock().unwrap();
    let idx = data.expenses.iter().position(|e| e.id == id)
        .ok_or_else(|| "Expense not found.".to_string())?;
    let original_ts = data.expenses[idx].created_at();
    let split = Split::new_equal(participants).map_err(|e| e)?;
    data.expenses[idx] = Expense::new(id, &description, amount, paid_by, group_id, split);
    Ok(())
}

#[tauri::command]
fn delete_expense(id: u64, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    let before = data.expenses.len();
    data.expenses.retain(|e| e.id != id);
    if data.expenses.len() == before {
        return Err("Expense not found.".into());
    }
    Ok(())
}

#[tauri::command]
fn list_expenses(state: State<AppState>) -> Vec<ExpenseDto> {
    let data = state.0.lock().unwrap();
    data.expenses.iter().rev().map(|e| expense_to_dto(e, &data)).collect()
}

// PAYMENTS 

#[tauri::command]
fn record_payment(
    from_id: u64,
    to_id: u64,
    amount: f64,
    group_id: Option<u64>,
    state: State<AppState>,
) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    if !data.users.iter().any(|u| u.id == from_id) {
        return Err("Sender not found".to_string());
    }
    if !data.users.iter().any(|u| u.id == to_id) {
        return Err("Recipient not found".to_string());
    }
    if let Some(gid) = group_id {
        if !data.groups.iter().any(|g| g.id == gid) {
            return Err("Group not found".to_string());
        }
    }
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    if from_id == to_id {
        return Err("Sender and recipient must be different".to_string());
    }

    let allocations: Vec<(Option<u64>, f64)> = if group_id.is_some() {
        vec![(group_id, amount)]
    } else {
        let pair_groups: Vec<u64> = data.groups.iter()
            .filter(|g| g.members().contains(&from_id) && g.members().contains(&to_id))
            .map(|g| g.id)
            .collect();

        let mut contexts: Vec<(Option<u64>, f64)> = Vec::new();

        let untagged_debt = Balance::pair_debt_in_context(&data.expenses, &data.payments, from_id, to_id, None);
        if untagged_debt > 0.0 {
            contexts.push((None, untagged_debt));
        }
        for gid in pair_groups {
            let debt = Balance::pair_debt_in_context(&data.expenses, &data.payments, from_id, to_id, Some(gid));
            if debt > 0.0 {
                contexts.push((Some(gid), debt));
            }
        }

        if contexts.is_empty() {
            vec![(None, amount)]
        } else {
            let total_debt: f64 = contexts.iter().map(|(_, d)| d).sum();
            let to_distribute = amount.min(total_debt);
            let remainder = amount - to_distribute;

            let mut allocs: Vec<(Option<u64>, f64)> = contexts.iter()
                .map(|(gid, debt)| {
                    let alloc = (to_distribute * (debt / total_debt) * 100.0).round() / 100.0;
                    (*gid, alloc)
                })
                .collect();

            let allocated_sum: f64 = allocs.iter().map(|(_, a)| a).sum();
            let drift = to_distribute - allocated_sum;
            if !allocs.is_empty() && drift.abs() > 0.001 {
                let last_idx = allocs.len() - 1;
                allocs[last_idx].1 = ((allocs[last_idx].1 + drift) * 100.0).round() / 100.0;
            }

            if remainder > 0.001 {
                if let Some(idx) = allocs.iter().position(|(g, _)| g.is_none()) {
                    allocs[idx].1 = ((allocs[idx].1 + remainder) * 100.0).round() / 100.0;
                } else {
                    allocs.push((None, (remainder * 100.0).round() / 100.0));
                }
            }

            allocs
        }
    };

    for (gid, alloc_amount) in allocations {
        if alloc_amount < 0.01 { continue; }
        data.next_payment_id += 1;
        let id = data.next_payment_id;
        let payment = Payment::new(id, from_id, to_id, alloc_amount, gid)?;
        data.payments.push(payment);
    }

    Ok(())
}

#[tauri::command]
fn delete_payment(id: u64, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    let before = data.payments.len();
    data.payments.retain(|p| p.id != id);
    if data.payments.len() == before {
        return Err("Payment not found".to_string());
    }
    Ok(())
}

#[tauri::command]
fn list_payments(state: State<AppState>) -> Vec<PaymentDto> {
    let data = state.0.lock().unwrap();
    data.payments.iter().rev().map(|p| payment_to_dto(p, &data)).collect()
}

#[tauri::command]
fn list_payments_for_friend(friend_id: u64, state: State<AppState>) -> Vec<PaymentDto> {
    let data = state.0.lock().unwrap();
    data.payments.iter()
        .filter(|p| p.from_id() == friend_id || p.to_id() == friend_id)
        .rev()
        .map(|p| payment_to_dto(p, &data))
        .collect()
}

#[tauri::command]
fn list_payments_for_group(group_id: u64, state: State<AppState>) -> Vec<PaymentDto> {
    let data = state.0.lock().unwrap();
    data.payments.iter()
        .filter(|p| p.group_id() == Some(group_id))
        .rev()
        .map(|p| payment_to_dto(p, &data))
        .collect()
}

// BALANCES 

#[tauri::command]
fn get_balances(state: State<AppState>) -> Vec<BalanceDto> {
    let data = state.0.lock().unwrap();
    let balances = Balance::balances_with_payments(&data.expenses, &data.payments);
    let mut result: Vec<BalanceDto> = data.users.iter().map(|u| BalanceDto {
        user_id: u.id,
        name: u.name().to_string(),
        amount: balances.get(&u.id).copied().unwrap_or(0.0),
    }).collect();
    result.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal));
    result
}

#[tauri::command]
fn simplify(state: State<AppState>) -> Vec<SettlementDto> {
    let data = state.0.lock().unwrap();
    let balances = Balance::balances_with_payments(&data.expenses, &data.payments);
    simplify_debts(&balances).iter().map(|d| SettlementDto {
        from: name_of(&data, d.from()),
        to: name_of(&data, d.to()),
        amount: d.amount(),
    }).collect()
}

// CURRENT USER 

#[tauri::command]
fn get_current_user(state: State<AppState>) -> Option<FriendDto> {
    let data = state.0.lock().unwrap();
    let my_id = data.current_user_id?;
    data.users.iter()
        .find(|u| u.id == my_id)
        .map(|u| {
            let pairwise = Balance::pairwise_balances(&data.expenses, &data.payments, my_id);
            let total: f64 = pairwise.values().sum();
            FriendDto {
                id: u.id,
                name: u.name().to_string(),
                balance: total,
            }
        })
}

#[tauri::command]
fn set_current_user(user_id: u64, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    if !data.users.iter().any(|u| u.id == user_id) {
        return Err("User not found".to_string());
    }
    data.current_user_id = Some(user_id);
    Ok(())
}

#[tauri::command]
fn clear_current_user(state: State<AppState>) {
    let mut data = state.0.lock().unwrap();
    data.current_user_id = None;
}

// ENTRY POINT 

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(AppData::default())))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_friend, list_friends,
            create_group, list_groups,
            add_expense, update_expense, delete_expense, list_expenses,
            get_balances, simplify,
            list_expenses_for_friend, friend_breakdown,
            list_expenses_for_group, group_balances, simplify_group,
            delete_group, add_members_to_group, remove_member_from_group,
            get_current_user, set_current_user, clear_current_user,
            record_payment, delete_payment, list_payments,
            list_payments_for_friend, list_payments_for_group,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}