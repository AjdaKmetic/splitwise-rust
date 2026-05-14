use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use settlemate_rust::{
    models::{expense::Expense, user::User},
    services::{
        balance::calculate_balances,
        simplify::simplify_debts,
        split::Split,
    },
};

struct GroupData {
    id: u64,
    name: String,
    member_ids: Vec<u64>,
}

#[derive(Default)]
struct AppData {
    users: Vec<User>,
    groups: Vec<GroupData>,      
    expenses: Vec<Expense>,
    next_user_id: u64,
    next_group_id: u64,
    next_expense_id: u64,
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
}

#[derive(Serialize, Clone)]
struct BalanceDto {
    user_id: u64, 
    name: String,
    amount: f64,
}

#[derive(Serialize)]
struct SettlementDto { from: String, to: String, amount: f64 }

fn name_of(data: &AppData, id: u64) -> String {
    data.users.iter()
        .find(|u| u.id == id)
        .map(|u| u.name().to_string())
        .unwrap_or_else(|| format!("?{id}"))
}

fn group_name_of(data: &AppData, id: u64) -> Option<String> {
    data.groups.iter().find(|g| g.id == id).map(|g| g.name.clone())
}

fn compute_pairwise_balances(expenses: &[Expense], my_id: u64) -> std::collections::HashMap<u64, f64> {
    let mut result: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    for expense in expenses {
        let amount = expense.amount();
        let paid_by = expense.paid_by();
        let shares: std::collections::HashMap<u64, f64> = match expense.splits() {
            Split::Equal(ids) => {
                if ids.is_empty() { continue; }
                let share = amount / ids.len() as f64;
                ids.iter().map(|&id| (id, share)).collect()
            },
            Split::Exact(pairs) => pairs.iter().cloned().collect(),
        };
        let my_share = shares.get(&my_id).copied().unwrap_or(0.0);
        if paid_by == my_id {
            for (&p_id, &share) in &shares {
                if p_id == my_id { continue; }
                *result.entry(p_id).or_insert(0.0) += share;
            }
        } else if shares.contains_key(&my_id) {
            *result.entry(paid_by).or_insert(0.0) -= my_share;
        }
    }
    result
}

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
            let pairwise = compute_pairwise_balances(&data.expenses, my_id);
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
            let balances = calculate_balances(&data.expenses);
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
fn create_group(name: String, member_ids: Vec<u64>, state: State<AppState>) -> Result<GroupDto, String> {
    if name.trim().is_empty() { return Err("Group name is required.".into()); }
    if member_ids.is_empty() { return Err("Select at least one member.".into()); }
    let mut data = state.0.lock().unwrap();
    data.next_group_id += 1;
    let id = data.next_group_id;
    data.groups.push(GroupData { id, name: name.clone(), member_ids: member_ids.clone() });
    let members: Vec<String> = member_ids.iter().map(|id| name_of(&data, *id)).collect();
    Ok(GroupDto { id, name, member_ids, members, expense_count: 0, has_outstanding: false })
}

#[tauri::command]
fn list_groups(state: State<AppState>) -> Vec<GroupDto> {
    let data = state.0.lock().unwrap();
    data.groups.iter().map(|g| {
        let group_expenses: Vec<Expense> = data.expenses.iter()
            .filter(|e| e.group_id() == Some(g.id))
            .cloned()
            .collect();
        let balances = calculate_balances(&group_expenses);
        let has_outstanding = balances.values().any(|&v| v.abs() > 0.01);
        GroupDto {
            id: g.id,
            name: g.name.clone(),
            member_ids: g.member_ids.clone(),
            members: g.member_ids.iter().map(|&id| name_of(&data, id)).collect(),
            expense_count: group_expenses.len(),
            has_outstanding,
        }
    }).collect()
}

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
    data.expenses.push(Expense::new(
        id, &description, amount, paid_by, group_id, Split::Equal(participants),
    ));
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
    data.expenses[idx] = Expense::new(
        id, &description, amount, paid_by, group_id, Split::Equal(participants),
    );
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
    data.expenses.iter().rev().map(|e| {
        let gid = e.group_id();
        let (participants, participant_ids): (Vec<String>, Vec<u64>) = match e.splits() {
            Split::Equal(ids) => ids.iter().map(|&id| (name_of(&data, id), id)).unzip(),
            Split::Exact(pairs) => pairs.iter().map(|(id, _)| (name_of(&data, *id), *id)).unzip(),
        };
        ExpenseDto {
            id: e.id,
            description: e.description().to_string(),
            amount: e.amount(),
            paid_by: name_of(&data, e.paid_by()),
            paid_by_id: e.paid_by(),
            participants,
            participant_ids,
            group_id: gid,
            group_name: gid.and_then(|id| group_name_of(&data, id)),
        }
    }).collect()
}

#[tauri::command]
fn get_balances(state: State<AppState>) -> Vec<BalanceDto> {
    let data = state.0.lock().unwrap();
    let balances = calculate_balances(&data.expenses);
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
    let balances = calculate_balances(&data.expenses);
    simplify_debts(&balances).iter().map(|d| SettlementDto {
        from: name_of(&data, d.from()),
        to: name_of(&data, d.to()),
        amount: d.amount(),
    }).collect()
}

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
            get_balances, simplify, list_expenses_for_friend, friend_breakdown, 
            list_expenses_for_group, group_balances, simplify_group,
            delete_group, add_members_to_group, remove_member_from_group,
            get_current_user, set_current_user, clear_current_user,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
        .map(|e| {
            let gid = e.group_id();
            let (participants, participant_ids): (Vec<String>, Vec<u64>) = match e.splits() {
                Split::Equal(ids) => ids.iter().map(|&id| (name_of(&data, id), id)).unzip(),
                Split::Exact(pairs) => pairs.iter().map(|(id, _)| (name_of(&data, *id), *id)).unzip(),
            };
            ExpenseDto {
                id: e.id,
                description: e.description().to_string(),
                amount: e.amount(),
                paid_by: name_of(&data, e.paid_by()),
                paid_by_id: e.paid_by(),
                participants, participant_ids,
                group_id: gid,
                group_name: gid.and_then(|id| group_name_of(&data, id)),
            }
        })
        .collect()
}

#[tauri::command]
fn friend_breakdown(friend_id: u64, state: State<AppState>) -> Vec<BalanceDto> {
    use std::collections::HashMap;
    let data = state.0.lock().unwrap();
    let mut breakdown: HashMap<u64, f64> = HashMap::new();

    for e in &data.expenses {
        let participants: Vec<u64> = match e.splits() {
            Split::Equal(ids) => ids.clone(),
            Split::Exact(pairs) => pairs.iter().map(|(id, _)| *id).collect(),
        };
        let share = e.amount() / participants.len() as f64;
        let payer = e.paid_by();

        if payer == friend_id {
            for &p in &participants {
                if p != friend_id {
                    *breakdown.entry(p).or_insert(0.0) += share;
                }
            }
        } else if participants.contains(&friend_id) {
            *breakdown.entry(payer).or_insert(0.0) -= share;
        }
    }

    breakdown.into_iter()
        .filter(|(_, amt)| amt.abs() > 0.005)
        .map(|(id, amount)| BalanceDto { user_id: id, name: name_of(&data, id), amount })
        .collect()
}
#[tauri::command]
fn list_expenses_for_group(group_id: u64, state: State<AppState>) -> Vec<ExpenseDto> {
    let data = state.0.lock().unwrap();
    data.expenses.iter()
        .filter(|e| e.group_id() == Some(group_id))
        .rev()
        .map(|e| {
            let gid = e.group_id();
            let (participants, participant_ids): (Vec<String>, Vec<u64>) = match e.splits() {
                Split::Equal(ids) => ids.iter().map(|&id| (name_of(&data, id), id)).unzip(),
                Split::Exact(pairs) => pairs.iter().map(|(id, _)| (name_of(&data, *id), *id)).unzip(),
            };
            ExpenseDto {
                id: e.id,
                description: e.description().to_string(),
                amount: e.amount(),
                paid_by: name_of(&data, e.paid_by()),
                paid_by_id: e.paid_by(),
                participants, participant_ids,
                group_id: gid,
                group_name: gid.and_then(|id| group_name_of(&data, id)),
            }
        })
        .collect()
}

#[tauri::command]
fn group_balances(group_id: u64, state: State<AppState>) -> Vec<BalanceDto> {
    let data = state.0.lock().unwrap();
    let group_expenses: Vec<Expense> = data.expenses.iter()
        .filter(|e| e.group_id() == Some(group_id))
        .cloned()
        .collect();
    let balances = calculate_balances(&group_expenses);
    let member_ids: Vec<u64> = data.groups.iter()
        .find(|g| g.id == group_id)
        .map(|g| g.member_ids.clone())
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
    let balances = calculate_balances(&group_expenses);
    simplify_debts(&balances).iter().map(|d| SettlementDto {
        from: name_of(&data, d.from()),
        to: name_of(&data, d.to()),
        amount: d.amount(),
    }).collect()
}

#[tauri::command]
fn delete_group(group_id: u64, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    if !data.groups.iter().any(|g| g.id == group_id) {
        return Err("Group not found".to_string());
    }
    data.groups.retain(|g| g.id != group_id);
    data.expenses.retain(|e| e.group_id() != Some(group_id));
    Ok(())
}

#[tauri::command]
fn add_members_to_group(group_id: u64, member_ids: Vec<u64>, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    let group = data.groups.iter_mut().find(|g| g.id == group_id)
        .ok_or_else(|| "Group not found".to_string())?;
    for id in member_ids {
        if !group.member_ids.contains(&id) {
            group.member_ids.push(id);
        }
    }
    Ok(())
}

#[tauri::command]
fn remove_member_from_group(group_id: u64, user_id: u64, state: State<AppState>) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();
    let group = data.groups.iter_mut().find(|g| g.id == group_id)
        .ok_or_else(|| "Group not found".to_string())?;
    if group.member_ids.len() <= 1 {
        return Err("Cannot remove the last member of a group".to_string());
    }
    group.member_ids.retain(|&id| id != user_id);
    Ok(())
}

#[tauri::command]
fn get_current_user(state: State<AppState>) -> Option<FriendDto> {
    let data = state.0.lock().unwrap();
    let my_id = data.current_user_id?;
    data.users.iter()
        .find(|u| u.id == my_id)
        .map(|u| {
            let pairwise = compute_pairwise_balances(&data.expenses, my_id);
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