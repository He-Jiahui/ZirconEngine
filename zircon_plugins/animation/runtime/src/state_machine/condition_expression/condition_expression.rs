use zircon_runtime::core::framework::animation::AnimationTransitionConditionAsset;

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionExpression {
    Condition(AnimationTransitionConditionAsset),
    All(Box<[ConditionExpression]>),
    Any(Box<[ConditionExpression]>),
    Not(Box<ConditionExpression>),
}

impl ConditionExpression {
    pub fn condition(condition: AnimationTransitionConditionAsset) -> Self {
        Self::Condition(condition)
    }

    pub fn all(expressions: impl IntoIterator<Item = ConditionExpression>) -> Self {
        Self::All(expressions.into_iter().collect())
    }

    pub fn any(expressions: impl IntoIterator<Item = ConditionExpression>) -> Self {
        Self::Any(expressions.into_iter().collect())
    }

    pub fn not(expression: ConditionExpression) -> Self {
        Self::Not(Box::new(expression))
    }
}
