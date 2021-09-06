#![warn(clippy::all)]
#![deny(clippy::correctness)]
#![forbid(unsafe_code)]

use crate::ty::{Env, Subs, VarState};
use hir::{
    expr::{Bound, Element, ElementKind, Expr, Literal, PlaceExpr, Range, Tag},
    statement::Statement,
};
use std::iter::once;

mod ty;

pub use crate::ty::{
    cons::{Cons, RowedType},
    MutType, Type, TypeError, Var,
};

struct TypedExpr<'a> {
    ty: Type<'a>,
    expr: Expr<'a, Type<'a>>,
}
fn infer_literal<'a>(literal: Literal) -> Type<'a> {
    let cons = match literal {
        Literal::Void => Cons::Unit,
        Literal::True | Literal::False => Cons::Bool,
        Literal::UInt(_) | Literal::Float(_) => Cons::Num,
    };
    Type::Cons(cons)
}
fn infer_expr<'a>(
    expr: Expr<'a, ()>,
    var_state: &mut VarState<'a>,
    env: &Env<'a>,
) -> Result<(Subs<'a>, TypedExpr<'a>), TypeError> {
    match expr {
        Expr::Literal(literal) => Ok((
            Subs::new(),
            TypedExpr {
                ty: infer_literal(literal),
                expr: Expr::Literal(literal),
            },
        )),
        Expr::Place(PlaceExpr::Var(var)) => match env.get(Var { name: var, id: 0 }) {
            Some(scheme) => Ok((
                Subs::new(),
                TypedExpr {
                    ty: scheme.instantiate(var_state)?,
                    expr: Expr::Place(PlaceExpr::Var(var)),
                },
            )),
            None => Err(TypeError::UnboundVar),
        },
        Expr::Array(elements) => {
            let mut subs = Subs::new();
            let mut typed_elements = Vec::new();
            let mut ty_var = Type::Var(var_state.new_var());
            let mut arr_ty = Type::Cons(Cons::Array(Box::new(ty_var.clone())));
            for element in Vec::from(elements) {
                let (more_subs, typed_expr) = infer_expr(element.expr, var_state, env)?;
                typed_elements.push(Element {
                    kind: element.kind,
                    expr: typed_expr.expr,
                });
                subs.compose_with(more_subs)?;
                let unify_to = match element.kind {
                    ElementKind::Splat => arr_ty.clone(),
                    ElementKind::Element => ty_var.clone(),
                };
                let arr_subs = unify_to.unify_with(typed_expr.ty, var_state)?;
                ty_var.substitute(&arr_subs)?;
                arr_ty.substitute(&arr_subs)?;
                // subs.compose_with(arr_subs)?;
            }
            Ok((
                subs,
                TypedExpr {
                    ty: arr_ty,
                    expr: Expr::Array(typed_elements.into()),
                },
            ))
        }
        Expr::ArrayRange(range) => {
            let mut subs = Subs::new();
            let left = match range.left {
                Some(bound) => {
                    let (more_subs, typed) = infer_expr(*bound.expr, var_state, env)?;
                    subs.compose_with(more_subs)?;
                    let more_subs = Type::Cons(Cons::Num).unify_with(typed.ty, var_state)?;
                    subs.compose_with(more_subs)?;
                    Some(Bound {
                        kind: bound.kind,
                        expr: Box::new(typed.expr),
                    })
                }
                None => None,
            };
            let right = match range.right {
                Some(bound) => {
                    let (more_subs, typed) = infer_expr(*bound.expr, var_state, env)?;
                    subs.compose_with(more_subs)?;
                    let more_subs = Type::Cons(Cons::Num).unify_with(typed.ty, var_state)?;
                    subs.compose_with(more_subs)?;
                    Some(Bound {
                        kind: bound.kind,
                        expr: Box::new(typed.expr),
                    })
                }
                None => None,
            };
            Ok((
                subs,
                TypedExpr {
                    ty: Type::Cons(Cons::Array(Box::new(Type::Cons(Cons::Num)))),
                    expr: Expr::ArrayRange(Range { left, right }),
                },
            ))
        }
        Expr::Tag(tag) => {
            let mut subs = Subs::new();
            let (expr, ty) = match tag.expr {
                Some(expr) => {
                    let (more_subs, typed) = infer_expr(*expr, var_state, env)?;
                    subs.compose_with(more_subs)?;
                    (Some(typed.expr), typed.ty)
                }
                None => (None, Type::Cons(Cons::Unit)),
            };
            Ok((
                subs,
                TypedExpr {
                    ty: Type::Cons(Cons::Union(RowedType {
                        fields: once((tag.tag, ty)).collect(),
                        rest: Some(var_state.new_var()),
                    })),
                    expr: Expr::Tag(Tag {
                        tag: tag.tag,
                        expr: expr.map(Box::new),
                    }),
                },
            ))
        }
        _ => todo!(),
    }
}
pub fn infer(statements: Vec<Statement<()>>) -> Result<Vec<Statement<Type>>, TypeError> {
    todo!()
}
pub fn test_infer(expr: Expr<()>) -> Result<Type, TypeError> {
    let (subs, typed_expr) = infer_expr(expr, &mut VarState::new(), &Env::new())?;
    let mut ty = typed_expr.ty;
    ty.substitute(&subs)?;
    Ok(ty)
}
