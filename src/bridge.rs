
pub enum Value<T> {
    /// Number values represent floating-point numbers like 37 or -9.25.
    /// 
    /// see: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number
    Number(f32),
    /// The String object is used to represent and manipulate a sequence of characters.
    /// 
    /// see: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String
    String(String),
    /// Boolean values can be one of two values: true or false, representing the truth value of a logical proposition.
    /// 
    /// see: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean
    Boolean(bool),
    /// The Function object provides methods for functions. In JavaScript, every function is actually a Function object.
    ///
    /// see: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function
    Function(T),
}
