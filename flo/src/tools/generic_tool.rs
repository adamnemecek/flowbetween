use super::tool_trait::*;
use super::tool_input::*;
use super::tool_action::*;
use super::super::viewmodel::*;

use animation::*;

use futures::*;

use std::any::*;
use std::sync::*;
use std::marker::PhantomData;

///
/// Trait implemented by FlowBetween tools
/// 
/// FloTools eliminate the need to know what the tool data structure stores.
/// 
pub trait FloTool<Anim: Animation> : Tool2<GenericToolData, Anim> {
}

///
/// The generic tool is used to convert a tool that uses a specific data type
/// to one that uses a standard data type. This makes it possible to use tools
/// without needing to know their underlying implementation.
/// 
/// A generic tool is typically used as its underlying Tool trait, for example
/// in an `Arc` reference.
/// 
pub struct GenericTool<ToolData: Send+'static, Anim: Animation, UnderlyingTool: Tool2<ToolData, Anim>> {
    /// The tool that this makes generic
    tool: UnderlyingTool,

    // Phantom data for the tool trait parameters
    phantom_anim: PhantomData<Anim>,
    phantom_tooldata: PhantomData<Mutex<ToolData>>
}

///
/// The data structure storing the generic tool data
/// 
pub struct GenericToolData(Box<Any+Send>);

///
/// Converts a tool to a generic tool
/// 
pub trait ToFloTool<Anim: Animation, ToolData: Send+'static> {
    ///
    /// Converts this object to a generic tool reference
    /// 
    fn to_flo_tool(self) -> Arc<FloTool<Anim>>;
}

impl GenericToolData {
    ///
    /// Converts an action to generic tool data
    /// 
    fn convert_action_to_generic<ToolData: 'static+Send+Sync>(action: ToolAction<ToolData>) -> ToolAction<GenericToolData> {
        use self::ToolAction::*;

        match action {
            Data(data)              => Data(GenericToolData(Box::new(Arc::new(data)))),
            Edit(edit)              => Edit(edit),
            BrushPreview(preview)   => BrushPreview(preview)
        }
    }

    ///
    /// Converts to a refernece of the specified type if possible
    /// 
    fn convert_ref<ToolData: 'static+Send>(&self) -> Option<Arc<ToolData>> {
        self.0.downcast_ref().cloned()
    }

    ///
    /// Converts an input value from generic tool data  to specific tool data
    /// 
    fn convert_input_from_generic<ToolData: 'static+Send>(input: ToolInput<GenericToolData>) -> Option<ToolInput<ToolData>> {
        use self::ToolInput::*;

        match input {
            Data(ref data)      => data.convert_ref().map(|data| Data(data)),
            PaintDevice(device) => Some(PaintDevice(device)),
            Paint(paint)        => Some(Paint(paint))
        }
    }
}

impl<ToolData: Send+'static, Anim: Animation, UnderlyingTool: Tool2<ToolData, Anim>> From<UnderlyingTool> for GenericTool<ToolData, Anim, UnderlyingTool> {
    fn from(tool: UnderlyingTool) -> GenericTool<ToolData, Anim, UnderlyingTool> {
        GenericTool {
            tool:               tool,
            phantom_anim:       PhantomData,
            phantom_tooldata:   PhantomData
        }
    }
}

impl<ToolData: Send+Sync+'static, Anim: Animation, UnderlyingTool: Tool2<ToolData, Anim>> Tool2<GenericToolData, Anim> for GenericTool<ToolData, Anim, UnderlyingTool> {
    fn tool_name(&self) -> String {
        self.tool.tool_name()
    }

    fn image_name(&self) -> String {
        self.tool.image_name()        
    }

    fn menu_controller_name(&self) -> String {
        self.tool.menu_controller_name()
    }

    fn actions_for_model(&self, model: Arc<AnimationViewModel<Anim>>) -> Box<Stream<Item=ToolAction<GenericToolData>, Error=()>> {
        // Map the underlying actions to generic actions
        Box::new(self.tool.actions_for_model(model)
            .map(|action| GenericToolData::convert_action_to_generic(action)))
    }

    fn actions_for_input<'a>(&'a self, data: Option<Arc<GenericToolData>>, input: Box<'a+Iterator<Item=ToolInput<GenericToolData>>>) -> Box<'a+Iterator<Item=ToolAction<GenericToolData>>> {
        // Generic data items from other tools don't generate data for this tool
        let data    = data.and_then(|data| data.convert_ref());
        let input   = Box::new(input
            .map(|input_item|       GenericToolData::convert_input_from_generic(input_item))
            .filter(|maybe_data|    maybe_data.is_some())
            .map(|definitely_data|  definitely_data.unwrap()));

        // Map the actions back to generic actions
        Box::new(self.tool.actions_for_input(data, input)
            .map(|action| GenericToolData::convert_action_to_generic(action)))
    }
}

impl<ToolData: Send+Sync+'static, Anim: Animation, UnderlyingTool: Tool2<ToolData, Anim>> FloTool<Anim> for GenericTool<ToolData, Anim, UnderlyingTool> {
}

///
/// Converts any tool to its generic 'FloTool' equivalent
/// 
impl<Anim: 'static+Animation, ToolData: 'static+Send+Sync, T: 'static+Tool2<ToolData, Anim>> ToFloTool<Anim, ToolData> for T {
    fn to_flo_tool(self) -> Arc<FloTool<Anim>> {
        Arc::new(GenericTool::from(self))
    }
}

///
/// Equality so that tool objects can be referred to in bindings
/// 
impl<Anim: Animation> PartialEq for FloTool<Anim> {
    fn eq(&self, other: &FloTool<Anim>) -> bool {
        self.tool_name() == other.tool_name()
    }
}