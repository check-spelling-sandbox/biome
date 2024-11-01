use biome_analyze::{
    context::RuleContext, declare_lint_rule, ActionCategory, Ast, FixKind, Rule, RuleDiagnostic,
    RuleSource,
};
use biome_console::markup;
use biome_js_factory::make;
use biome_js_syntax::{AnyJsArrayElement, AnyJsExpression, JsArrayExpression, TriviaPieceKind};
use biome_rowan::{AstNode, AstNodeExt, AstSeparatedList, BatchMutationExt};

use crate::JsRuleAction;

declare_lint_rule! {
    /// Disallow sparse arrays
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// [1,,2]
    /// ```
    pub NoSparseArray {
        version: "1.0.0",
        name: "noSparseArray",
        language: "js",
        sources: &[RuleSource::Eslint("no-sparse-arrays")],
        recommended: true,
        fix_kind: FixKind::Unsafe,
    }
}

impl Rule for NoSparseArray {
    type Query = Ast<JsArrayExpression>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let node = ctx.query();

        // We defer collect `JsHole` index until user want to apply code action.
        node.elements().iter().find_map(|element| {
            if matches!(element.ok()?, AnyJsArrayElement::JsArrayHole(_),) {
                Some(())
            } else {
                None
            }
        })
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        Some(RuleDiagnostic::new(rule_category!(),
            node.syntax().text_trimmed_range(),
markup! {
                "This "<Emphasis>"array"</Emphasis>" contains an "<Emphasis>"empty slot"</Emphasis>"."
            }
            .to_owned()
        ))
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<JsRuleAction> {
        let node = ctx.query();
        let mut mutation = ctx.root().begin();

        let mut final_array_element_list = node.elements();

        for (i, item) in final_array_element_list.iter().enumerate() {
            if matches!(item, Ok(AnyJsArrayElement::JsArrayHole(_))) {
                let undefine_indent = if i == 0 {
                    make::ident("undefined")
                } else {
                    make::ident("undefined")
                        .with_leading_trivia([(TriviaPieceKind::Whitespace, " ")])
                };
                let ident_expr =
                    make::js_identifier_expression(make::js_reference_identifier(undefine_indent));
                // Why we need to use `final_array_element_list.iter().nth(i)` instead of `item`, because every time we
                // call `replace_node` the previous iteration `item` is not the descent child of current `final_array_element_list` anymore.
                let n_element = final_array_element_list.iter().nth(i)?.ok()?;
                final_array_element_list = final_array_element_list.replace_node(
                    n_element,
                    AnyJsArrayElement::AnyJsExpression(AnyJsExpression::JsIdentifierExpression(
                        ident_expr,
                    )),
                )?;
            }
        }

        mutation.replace_node(
            node.clone(),
            make::js_array_expression(
                node.l_brack_token().ok()?,
                final_array_element_list,
                node.r_brack_token().ok()?,
            ),
        );

        Some(JsRuleAction::new(
            ActionCategory::QuickFix,
            ctx.metadata().applicability(),
            markup! { "Replace hole with undefined" }.to_owned(),
            mutation,
        ))
    }
}
