/// Evaluator for trigger script AST
///
/// Evaluates parsed trigger scripts in a sandboxed environment.
/// Executes actions and evaluates conditions against game state.

use super::{AstNode, BinaryOperator, TriggerContext, TriggerResult};
use super::{MAX_ACTIONS_PER_TRIGGER, MAX_MESSAGES_PER_TRIGGER};
use crate::tmush::errors::TinyMushError;
use crate::tmush::storage::TinyMushStore;

/// Value type for evaluation results
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(i64),
    Boolean(bool),
    Null,
}

impl Value {
    /// Convert value to boolean for condition evaluation
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
        }
    }
    
    /// Get string representation
    pub fn as_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

/// Evaluator state and execution
pub struct Evaluator<'a> {
    context: &'a mut TriggerContext,
    store: &'a TinyMushStore,
    messages: Vec<String>,
}

impl<'a> Evaluator<'a> {
    pub fn new(context: &'a mut TriggerContext, store: &'a TinyMushStore) -> Self {
        Self {
            context,
            store,
            messages: Vec::new(),
        }
    }
    
    /// Evaluate an AST node
    pub fn evaluate(&mut self, node: &AstNode) -> Result<Value, String> {
        // Check timeout
        if self.context.is_timed_out() {
            return Err("Execution timeout".to_string());
        }
        
        match node {
            AstNode::StringLiteral(s) => Ok(Value::String(s.clone())),
            
            AstNode::NumberLiteral(n) => Ok(Value::Number(*n)),
            
            AstNode::Variable(name) => self.evaluate_variable(name),
            
            AstNode::Action { name, args } => self.evaluate_action(name, args),
            
            AstNode::BinaryOp { op, left, right } => {
                self.evaluate_binary_op(*op, left, right)
            }
            
            AstNode::Ternary { condition, then_branch, else_branch } => {
                let cond_value = self.evaluate(condition)?;
                if cond_value.is_truthy() {
                    self.evaluate(then_branch)
                } else {
                    self.evaluate(else_branch)
                }
            }
            
            AstNode::Sequence(nodes) => {
                let mut last_value = Value::Null;
                for node in nodes {
                    last_value = self.evaluate(node)?;
                }
                Ok(last_value)
            }
        }
    }
    
    /// Evaluate a variable reference ($player, $object, etc.)
    fn evaluate_variable(&self, name: &str) -> Result<Value, String> {
        match name {
            "player" => Ok(Value::String(self.context.player_username.clone())),
            "player_name" => Ok(Value::String(self.context.player_name.clone())),
            "object" => Ok(Value::String(self.context.object_name.clone())),
            "object_id" => Ok(Value::String(self.context.object_id.clone())),
            "room" => Ok(Value::String(self.context.room_name.clone())),
            "room_id" => Ok(Value::String(self.context.room_id.clone())),
            _ => Err(format!("Unknown variable: ${}", name)),
        }
    }
    
    /// Evaluate a binary operator
    fn evaluate_binary_op(
        &mut self,
        op: BinaryOperator,
        left: &AstNode,
        right: &AstNode,
    ) -> Result<Value, String> {
        match op {
            BinaryOperator::And => {
                let left_val = self.evaluate(left)?;
                if !left_val.is_truthy() {
                    return Ok(Value::Boolean(false));
                }
                let right_val = self.evaluate(right)?;
                Ok(Value::Boolean(right_val.is_truthy()))
            }
            
            BinaryOperator::Or => {
                let left_val = self.evaluate(left)?;
                if left_val.is_truthy() {
                    return Ok(Value::Boolean(true));
                }
                let right_val = self.evaluate(right)?;
                Ok(Value::Boolean(right_val.is_truthy()))
            }
            
            BinaryOperator::Equal => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                Ok(Value::Boolean(left_val == right_val))
            }
            
            BinaryOperator::NotEqual => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                Ok(Value::Boolean(left_val != right_val))
            }
            
            BinaryOperator::Greater => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                match (left_val, right_val) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
                    _ => Err("Greater than comparison requires numbers".to_string()),
                }
            }
            
            BinaryOperator::Less => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                match (left_val, right_val) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
                    _ => Err("Less than comparison requires numbers".to_string()),
                }
            }
            
            BinaryOperator::GreaterEqual => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                match (left_val, right_val) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
                    _ => Err("Greater or equal comparison requires numbers".to_string()),
                }
            }
            
            BinaryOperator::LessEqual => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                match (left_val, right_val) {
                    (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
                    _ => Err("Less or equal comparison requires numbers".to_string()),
                }
            }
        }
    }
    
    /// Evaluate an action (function call)
    fn evaluate_action(&mut self, name: &str, args: &[AstNode]) -> Result<Value, String> {
        // Check action limit
        if !self.context.can_execute_action() {
            return Err(format!(
                "Action limit reached ({} max)",
                MAX_ACTIONS_PER_TRIGGER
            ));
        }
        
        self.context.increment_action();
        
        match name {
            "message" => self.action_message(args),
            "message_room" => self.action_message_room(args),
            "has_item" => self.condition_has_item(args),
            "has_quest" => self.condition_has_quest(args),
            "flag_set" => self.condition_flag_set(args),
            "room_flag" => self.condition_room_flag(args),
            "current_room" => Ok(Value::String(self.context.room_id.clone())),
            "random_chance" => self.condition_random_chance(args),
            
            // Actions that modify game state (will implement in Phase 4)
            "teleport" => self.action_teleport(args),
            "grant_item" => self.action_grant_item(args),
            "consume" => self.action_consume(args),
            "heal" => self.action_heal(args),
            "unlock_exit" => self.action_unlock_exit(args),
            "lock_exit" => self.action_lock_exit(args),
            
            _ => Err(format!("Unknown action: {}", name)),
        }
    }
    
    /// Action: Send message to player
    fn action_message(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("message() expects 1 argument, got {}", args.len()));
        }
        
        if !self.context.can_send_message() {
            return Err(format!(
                "Message limit reached ({} max)",
                MAX_MESSAGES_PER_TRIGGER
            ));
        }
        
        let text = self.evaluate(&args[0])?;
        let msg = self.substitute_variables(&text.as_string());
        
        self.messages.push(msg);
        self.context.increment_message();
        
        Ok(Value::Boolean(true))
    }
    
    /// Action: Send message to entire room
    fn action_message_room(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("message_room() expects 1 argument, got {}", args.len()));
        }
        
        if !self.context.can_send_message() {
            return Err(format!(
                "Message limit reached ({} max)",
                MAX_MESSAGES_PER_TRIGGER
            ));
        }
        
        let text = self.evaluate(&args[0])?;
        let msg = format!("🔊 {}", self.substitute_variables(&text.as_string()));
        
        self.messages.push(msg);
        self.context.increment_message();
        
        Ok(Value::Boolean(true))
    }
    
    /// Condition: Check if player has item
    fn condition_has_item(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("has_item() expects 1 argument, got {}", args.len()));
        }
        
        let _item_id = self.evaluate(&args[0])?;
        
        // TODO: Actually check player inventory (Phase 4)
        // For now, return false
        Ok(Value::Boolean(false))
    }
    
    /// Condition: Check if player has quest
    fn condition_has_quest(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("has_quest() expects 1 argument, got {}", args.len()));
        }
        
        let _quest_id = self.evaluate(&args[0])?;
        
        // TODO: Actually check player quests (Phase 4)
        // For now, return false
        Ok(Value::Boolean(false))
    }
    
    /// Condition: Check if object/room has flag
    fn condition_flag_set(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("flag_set() expects 1 argument, got {}", args.len()));
        }
        
        let _flag_name = self.evaluate(&args[0])?;
        
        // TODO: Actually check object flags (Phase 4)
        // For now, return false
        Ok(Value::Boolean(false))
    }
    
    /// Condition: Check if current room has flag
    fn condition_room_flag(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("room_flag() expects 1 argument, got {}", args.len()));
        }
        
        let _flag_name = self.evaluate(&args[0])?;
        
        // TODO: Actually check room flags (Phase 4)
        // For now, return false
        Ok(Value::Boolean(false))
    }
    
    /// Condition: Random chance (percentage 0-100)
    fn condition_random_chance(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("random_chance() expects 1 argument, got {}", args.len()));
        }
        
        let chance = self.evaluate(&args[0])?;
        
        match chance {
            Value::Number(n) if (0..=100).contains(&n) => {
                use rand::Rng;
                let roll = rand::thread_rng().gen_range(1..=100);
                Ok(Value::Boolean(roll <= n))
            }
            _ => Err("random_chance() expects a number 0-100".to_string()),
        }
    }
    
    // Stub actions for Phase 4
    
    fn action_teleport(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("teleport() expects 1 argument, got {}", args.len()));
        }
        let _room_id = self.evaluate(&args[0])?;
        // TODO: Implement in Phase 4
        Ok(Value::Boolean(true))
    }
    
    fn action_grant_item(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("grant_item() expects 1 argument, got {}", args.len()));
        }
        let _item_id = self.evaluate(&args[0])?;
        // TODO: Implement in Phase 4
        Ok(Value::Boolean(true))
    }
    
    fn action_consume(&self, _args: &[AstNode]) -> Result<Value, String> {
        // TODO: Implement in Phase 4
        Ok(Value::Boolean(true))
    }
    
    fn action_heal(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("heal() expects 1 argument, got {}", args.len()));
        }
        let _amount = self.evaluate(&args[0])?;
        // TODO: Implement in Phase 4
        Ok(Value::Boolean(true))
    }
    
    fn action_unlock_exit(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("unlock_exit() expects 1 argument, got {}", args.len()));
        }
        let _direction = self.evaluate(&args[0])?;
        // TODO: Implement in Phase 4
        Ok(Value::Boolean(true))
    }
    
    fn action_lock_exit(&mut self, args: &[AstNode]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("lock_exit() expects 1 argument, got {}", args.len()));
        }
        let _direction = self.evaluate(&args[0])?;
        // TODO: Implement in Phase 4
        Ok(Value::Boolean(true))
    }
    
    /// Substitute variables in strings
    fn substitute_variables(&self, text: &str) -> String {
        text.replace("$player", &self.context.player_username)
            .replace("$player_name", &self.context.player_name)
            .replace("$object", &self.context.object_name)
            .replace("$object_id", &self.context.object_id)
            .replace("$room", &self.context.room_name)
            .replace("$room_id", &self.context.room_id)
    }
    
    /// Get collected messages
    pub fn messages(&self) -> &[String] {
        &self.messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::types::{PlayerRecord, ObjectRecord, RoomRecord};
    use crate::tmush::types::{ObjectOwner, RoomOwner, RoomVisibility};
    use crate::tmush::trigger::TriggerContext;
    use chrono::Utc;
    use tempfile::TempDir;
    
    fn create_test_setup() -> (TempDir, TinyMushStore, TriggerContext) {
        let temp_dir = TempDir::new().unwrap();
        let store = TinyMushStore::open(temp_dir.path()).unwrap();
        
        let player = PlayerRecord::new("test_player", "Test Player", "test_room");
        
        let object = ObjectRecord {
            id: "test_object".to_string(),
            name: "Test Object".to_string(),
            description: "A test object".to_string(),
            owner: ObjectOwner::World,
            created_at: Utc::now(),
            weight: 1,
            currency_value: Default::default(),
            value: 0,
            takeable: true,
            usable: true,
            actions: Default::default(),
            flags: vec![],
            locked: false,
            ownership_history: vec![],
            schema_version: 1,
        };
        
        let room = RoomRecord {
            id: "test_room".to_string(),
            name: "Test Room".to_string(),
            short_desc: "A test room".to_string(),
            long_desc: "This is a test room".to_string(),
            owner: RoomOwner::World,
            created_at: Utc::now(),
            visibility: RoomVisibility::Public,
            exits: Default::default(),
            items: vec![],
            flags: vec![],
            max_capacity: 10,
            housing_filter_tags: vec![],
            locked: false,
            schema_version: 1,
        };
        
        let context = TriggerContext::new(&player, &object, &room);
        
        (temp_dir, store, context)
    }
    
    #[test]
    fn test_evaluate_literals() {
        let (_temp, store, mut context) = create_test_setup();
        let mut evaluator = Evaluator::new(&mut context, &store);
        
        let node = AstNode::StringLiteral("hello".to_string());
        let result = evaluator.evaluate(&node).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
        
        let node = AstNode::NumberLiteral(42);
        let result = evaluator.evaluate(&node).unwrap();
        assert_eq!(result, Value::Number(42));
    }
    
    #[test]
    fn test_evaluate_variables() {
        let (_temp, store, mut context) = create_test_setup();
        let mut evaluator = Evaluator::new(&mut context, &store);
        
        let node = AstNode::Variable("player".to_string());
        let result = evaluator.evaluate(&node).unwrap();
        assert_eq!(result, Value::String("test_player".to_string()));
    }
    
    #[test]
    fn test_evaluate_message() {
        let (_temp, store, mut context) = create_test_setup();
        let mut evaluator = Evaluator::new(&mut context, &store);
        
        let node = AstNode::Action {
            name: "message".to_string(),
            args: vec![AstNode::StringLiteral("Hello $player!".to_string())],
        };
        
        evaluator.evaluate(&node).unwrap();
        
        assert_eq!(evaluator.messages().len(), 1);
        assert_eq!(evaluator.messages()[0], "Hello test_player!");
    }
    
    #[test]
    fn test_evaluate_ternary() {
        let (_temp, store, mut context) = create_test_setup();
        let mut evaluator = Evaluator::new(&mut context, &store);
        
        // true ? "yes" : "no"
        let node = AstNode::Ternary {
            condition: Box::new(AstNode::NumberLiteral(1)),
            then_branch: Box::new(AstNode::StringLiteral("yes".to_string())),
            else_branch: Box::new(AstNode::StringLiteral("no".to_string())),
        };
        
        let result = evaluator.evaluate(&node).unwrap();
        assert_eq!(result, Value::String("yes".to_string()));
    }
    
    #[test]
    fn test_message_limit() {
        let (_temp, store, mut context) = create_test_setup();
        let mut evaluator = Evaluator::new(&mut context, &store);
        
        // Send 3 messages (should succeed)
        for i in 0..3 {
            let node = AstNode::Action {
                name: "message".to_string(),
                args: vec![AstNode::StringLiteral(format!("Message {}", i))],
            };
            evaluator.evaluate(&node).unwrap();
        }
        
        // 4th message should fail
        let node = AstNode::Action {
            name: "message".to_string(),
            args: vec![AstNode::StringLiteral("Message 4".to_string())],
        };
        let result = evaluator.evaluate(&node);
        assert!(result.is_err());
    }
}
