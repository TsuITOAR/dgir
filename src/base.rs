trait Attachable:Sized{
    fn attach<E:Extendable>(self,e:&mut E)->&mut E{
        e.extend(self)
    }
}

trait Extendable:Sized{
    fn extend<A:Attachable>(&mut self,a:A)->&mut Self{
        unimplemented!();
    }
    fn adapt<A:Adapter>(&mut self,adapter:A)->&mut Self{
        todo!()
    }
    fn connect<T:Extendable,U>(&mut self,target:T)->U{
        unimplemented!();
    }
}

trait Adapter:Sized{
    fn reverse(self)->Self{
        unimplemented!();
    }
}

impl<T:Adapter> Attachable for T{}

/* trait Integrated:Sized{
    fn place()
} */

struct VectorInfo<P,A>{
    position:(P,P),
    direction:A,
    layer:i16,
    datatype:i16
}

struct Port<P=f64,A=f64,const N:usize=0>{
    vector_info:VectorInfo<P,A>,
    port:[;N]
}