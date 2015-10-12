use ::AbsContext;

pub trait Builder<'ctxt, 'init>: Sized {
    type Context: AbsContext + 'ctxt;
    type InitArgs: Sized + 'init;
    type Final: Buildable<'ctxt, 'init>;

    fn build(self) -> Self::Final;
}

pub trait Buildable<'ctxt, 'init>: Sized {
    type Builder: Builder<'ctxt, 'init>; 
   
    fn builder(
        ctxt: &'ctxt <Self::Builder as Builder<'ctxt, 'init>>::Context,
        init_args: <Self::Builder as Builder<'ctxt, 'init>>::InitArgs,
    ) -> Self::Builder;
}
