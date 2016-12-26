use chalk_parse::ast;
use formula::*;

use super::error::LowerResult;
use super::environment::LowerEnvironment;
use super::lower_application::LowerApplication;
use super::lower_clause::LowerClause;

pub trait LowerGoal<L> {
    fn lower_goal(&self, env: &mut LowerEnvironment) -> LowerResult<Goal<L>>;
}

impl LowerGoal<Application> for ast::Application {
    fn lower_goal(&self, env: &mut LowerEnvironment) -> LowerResult<Goal<Application>> {
        // collect the wildcards and bring them into scope
        let wildcards = self.count_wildcards();
        env.push_wildcards(wildcards);
        let application = self.lower_application(env)?;
        let goal = Goal::new(GoalData { kind: GoalKind::Leaf(application) });
        let goal = goal.in_exists(wildcards);
        env.pop_wildcards(wildcards);
        Ok(goal)
    }
}

impl LowerGoal<Application> for ast::Fact {
    fn lower_goal(&self, env: &mut LowerEnvironment) -> LowerResult<Goal<Application>> {
        match *self.data {
            ast::FactData::True => {
                Ok(Goal::new(GoalData { kind: GoalKind::True }))
            }

            ast::FactData::False => {
                Ok(Goal::new(GoalData { kind: GoalKind::False }))
            }

            ast::FactData::And(ref f1, ref f2) => {
                let c1 = f1.lower_goal(env)?;
                let c2 = f2.lower_goal(env)?;
                Ok(Goal::new(GoalData { kind: GoalKind::And(c1, c2) }))
            }

            ast::FactData::Not(ref f1) => {
                let c1 = f1.lower_goal(env)?;
                Ok(Goal::new(GoalData { kind: GoalKind::Not(c1) }))
            }

            ast::FactData::Or(ref f1, ref f2) => {
                let c1 = f1.lower_goal(env)?;
                let c2 = f2.lower_goal(env)?;
                Ok(Goal::new(GoalData { kind: GoalKind::Or(c1, c2) }))
            }

            ast::FactData::Implication(ref f1, ref f2) => {
                let condition = f1.lower_clause(env)?;
                let consequence = f2.lower_goal(env)?;
                Ok(Goal::new(GoalData {
                    kind: GoalKind::Implication(condition, consequence),
                }))
            }

            ast::FactData::ForAll(v, ref f1) => {
                env.push_bound_name(v);
                let c = f1.lower_goal(env)?;
                env.pop_bound_name();
                Ok(c.in_foralls(1))
            }

            ast::FactData::Exists(v, ref f1) => {
                env.push_bound_name(v);
                let c = f1.lower_goal(env)?;
                env.pop_bound_name();
                Ok(c.in_exists(1))
            }

            ast::FactData::IfThenElse(ref c, ref t, ref e) => {
                let c = c.lower_goal(env)?;
                let t = t.lower_goal(env)?;
                let e = e.lower_goal(env)?;
                Ok(Goal::new(GoalData {
                    kind: GoalKind::IfThenElse(c, t, e)
                }))
            }

            ast::FactData::Apply(ref appl) => {
                appl.lower_goal(env)
            }
        }
    }
}
