mod element;
mod iter;
mod node;
mod safe_element;
mod safe_node;

use std::sync::Arc;

pub use element::*;
pub use iter::*;
use layout::{
    BoxTree, FragmentTree,
    context::{ImageResolver, LayoutContext},
    display_list::StackingContextTree,
    dom::NodeExt,
};
use layout_api::{LayoutDamage, wrapper_traits::LayoutNode};
pub use node::*;
pub use safe_element::*;
pub use safe_node::*;
use script::layout_dom::LayoutNodeExt;
use servo_config::opts::DebugOptions;
use style::{selector_parser::RestyleDamage, stylist::Stylist};
use webrender_api::{BuiltDisplayList, PipelineId, units::LayoutSize};

pub type BlitzNode<'dom> = &'dom blitz_dom::Node;

pub struct LayoutOutput {
    pub box_tree: Option<Arc<BoxTree>>,
    pub fragment_tree: FragmentTree,
    pub stacking_context_tree: StackingContextTree,
    pub pipeline_id: PipelineId,
    pub display_list: BuiltDisplayList,
}

pub fn layout_and_build_display_list<'dom, T, U>(
    dirty_root: T,
    root: U,
    layout_context: LayoutContext,
    stylist: &Stylist,
    image_resolver: Arc<ImageResolver>,
    debug_options: &DebugOptions,
) -> LayoutOutput
where
    T: LayoutNode<'dom> + NodeExt<'dom> + LayoutNodeExt<'dom>,
    U: LayoutNode<'dom> + NodeExt<'dom> + LayoutNodeExt<'dom>,
{
    let mut box_tree: Option<Arc<BoxTree>> = None;
    let restyle_damage = RestyleDamage::RELAYOUT; // TODO: 
    let layout_damage: LayoutDamage = restyle_damage.into();

    if box_tree.is_none() || layout_damage.has_box_damage() {
        let mut build_box_tree = || {
            log::debug!("ran build_box_tree");
            if !BoxTree::update(&layout_context, dirty_root) {
                box_tree = Some(Arc::new(BoxTree::construct(&layout_context, root)));
            }
        };

        // TODO: run in parallel with rayon?
        build_box_tree();
    }

    assert!(box_tree.is_some());

    let viewport_size = stylist.device().au_viewport_size();
    let run_layout = || {
        box_tree
            .as_ref()
            .unwrap()
            .layout(&layout_context, viewport_size)
    };
    // TODO: run in parallel with rayon?
    let fragment_tree = run_layout();

    fragment_tree.calculate_scrollable_overflow();

    if debug_options.dump_flow_tree {
        fragment_tree.print();
    }

    let id = webrender_api::PipelineId::dummy();
    let first_reflow = true;

    let px_viewport_size = LayoutSize::new(
        viewport_size.width.to_f32_px(),
        viewport_size.height.to_f32_px(),
    );

    // build stacking context tree
    let mut stacking_context_tree = StackingContextTree::new(
        &fragment_tree,
        px_viewport_size,
        id,
        first_reflow,
        debug_options,
    );

    // Build display list
    let compositor_info = &mut stacking_context_tree.compositor_info;
    compositor_info.hit_test_info.clear();

    let mut webrender_display_list_builder =
        webrender_api::DisplayListBuilder::new(compositor_info.pipeline_id);
    webrender_display_list_builder.begin();

    if debug_options.dump_display_list {
        webrender_display_list_builder.dump_serialized_display_list();
    }

    let mut builder = layout::display_list::DisplayListBuilder {
        current_scroll_node_id: compositor_info.root_reference_frame_id,
        current_reference_frame_scroll_node_id: compositor_info.root_reference_frame_id,
        current_clip_id: layout::display_list::clip::ClipId::INVALID,
        webrender_display_list_builder: &mut webrender_display_list_builder,
        compositor_info,
        inspector_highlight: None,
        paint_body_background: true,
        clip_map: Default::default(),
        image_resolver,
        device_pixel_ratio: stylist.device().device_pixel_ratio(),
    };

    builder.add_all_spatial_nodes();

    for clip in stacking_context_tree.clip_store.0.iter() {
        builder.add_clip_to_display_list(clip);
    }

    stacking_context_tree
        .root_stacking_context
        .build_canvas_background_display_list(&mut builder, &fragment_tree);
    stacking_context_tree
        .root_stacking_context
        .build_display_list(&mut builder);

    builder.paint_dom_inspector_highlight();

    let (pipeline_id, display_list) = webrender_display_list_builder.end();

    LayoutOutput {
        box_tree,
        fragment_tree,
        stacking_context_tree,
        pipeline_id,
        display_list,
    }
}

#[cfg(test)]
mod tests {}
