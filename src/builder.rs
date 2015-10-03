use ::Context;

pub trait Builder<'a> {
    type InitArgs: 'a;
    type Final: Buildable<'a>;

    fn new(context: &'a mut Context, args: Self::InitArgs) -> Self;
    fn build(self) -> Self::Final;
}

pub trait Buildable<'a> {
    type Builder: Builder<'a>;
}
