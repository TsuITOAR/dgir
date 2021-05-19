trait Extendable:Sized{
    fn extend(&mut self)->&mut Self{
        unimplemented!();
    }
    fn adapt<A:Adapter>(&mut self,adapter:A)->&mut Self{
        todo!()
    }
    fn connect<T:Extendable,U:Extendable>(&mut self,target:T)->U{
        unimplemented!();
    }
}

trait Adapter:Sized{
    fn reverse(self)->Self{
        unimplemented!();
    }
}

trait Integrated:Sized{
    fn place()
}