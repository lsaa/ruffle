//! Special handling for AVM2 orphan objects

use crate::context::UpdateContext;
use crate::display_object::{
    DisplayObject, DisplayObjectWeak, TDisplayObject, TDisplayObjectContainer,
};
use crate::frame_lifecycle::FramePhase;
use gc_arena::{Collect, Mutation};
use std::rc::Rc;

/// The list of 'orphan' objects - these objects have no parent,
/// so we need to manually run their frames in `run_all_phases_avm2` to match
/// Flash's behavior. Clips are added to this list with `add_orphan_movie`.
/// and are removed automatically by `cleanup_dead_orphans`.
///
/// We store `DisplayObjectWeak`, since we don't want to keep these objects
/// alive if they would otherwise be garbage-collected. The movie will
/// stop ticking whenever garbage collection runs if there are no more
/// strong references around (this matches Flash's behavior).
#[derive(Collect)]
#[collect(no_drop)]
pub struct OrphanManager<'gc> {
    orphans: Rc<Vec<Orphan<'gc>>>,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
struct Orphan<'gc> {
    object: DisplayObjectWeak<'gc>,
    // False once timeline removal has retired the object itself.
    // Its children must still participate in the frame lifecycle.
    run_self: bool,
}

impl<'gc> OrphanManager<'gc> {
    fn orphans_mut(&mut self) -> &mut Vec<Orphan<'gc>> {
        Rc::make_mut(&mut self.orphans)
    }

    /// Adds a `MovieClip` to the orphan list. In AVM2, movies advance their
    /// frames even when they are not on a display list. Unfortunately,
    /// multiple SWFS rely on this behavior, so we need to match Flash's
    /// behavior. This should not be called manually - `movie_clip` will
    /// call it when necessary.
    pub fn add_orphan_obj(&mut self, dobj: DisplayObject<'gc>) {
        // Note: comparing pointers is correct because GcWeak keeps its allocation alive,
        // so the pointers can't overlap by accident.
        if let Some(orphan) = self
            .orphans_mut()
            .iter_mut()
            .find(|orphan| std::ptr::eq(orphan.object.as_ptr(), dobj.as_ptr()))
        {
            orphan.run_self = true;
        } else {
            self.orphans_mut().push(Orphan {
                object: dobj.downgrade(),
                run_self: true,
            });
        }
    }

    pub fn each_orphan_obj(
        context: &mut UpdateContext<'gc>,
        mut f: impl FnMut(DisplayObject<'gc>, &mut UpdateContext<'gc>),
    ) {
        // Clone the Rc before iterating over it. Any modifications must go through
        // `Rc::make_mut` in `orphan_objects_mut`, which will leave this `Rc` unmodified.
        // This ensures that any orphan additions/removals done by `f` will not affect
        // the iteration in this method.
        let orphan_objs = context.orphan_manager.orphans.clone();

        for orphan in orphan_objs.iter() {
            if let Some(dobj) = valid_orphan(orphan.object, context.gc()) {
                if orphan.run_self || dobj.placed_by_avm2_script() {
                    f(dobj, context);
                } else if let Some(container) = dobj.as_container() {
                    let children = container.iter_render_list();

                    if *context.frame_phase == FramePhase::Enter {
                        children.rev().for_each(|child| f(child, context));
                    } else {
                        children.for_each(|child| f(child, context));
                    }
                }
            }
        }
    }

    /// Called at the end of `run_all_phases_avm2` - removes any movies
    /// that have been garbage collected, or are no longer orphans
    /// (they've since acquired a parent).
    pub fn cleanup_dead_orphans(&mut self, mc: &Mutation<'gc>) {
        self.orphans_mut().retain_mut(|orphan| {
            let Some(dobj) = valid_orphan(orphan.object, mc) else {
                return false;
            };
            // All clips that become orphaned (have their parent removed, or start out with no parent)
            // get added to the orphan list. However, there's a distinction between clips
            // that are removed by a RemoveObject tag and clips removed from ActionScript.
            //
            // Clips removed by a RemoveObject tag run until the end of the frame, allowing
            // their final frame script to run with `this.parent == null`. After that, we
            // keep them on the list to process their children, but no longer run the
            // removed clips themselves.
            //
            // Clips removed from ActionScript continue running indefinitely while orphaned
            // (if there are no remaining strong references, they will eventually be
            // garbage collected).
            //
            // To distinguish these cases, we check `placed_by_avm2_script`. This flag is
            // set for objects constructed from ActionScript, and for objects moved around
            // in the timeline (add/remove child, swap depths) by ActionScript. A
            // RemoveObject tag only affects objects instantiated by the timeline that
            // have not been moved in the display list by ActionScript. Therefore, an
            // orphan with this flag set should continue running itself as well.
            orphan.run_self = dobj.placed_by_avm2_script();
            true
        });
    }
}

impl<'gc> Default for OrphanManager<'gc> {
    fn default() -> Self {
        Self {
            orphans: Rc::new(Vec::new()),
        }
    }
}

/// If the provided `DisplayObjectWeak` should have frames run, returns
/// Some(clip) with an upgraded `MovieClip`.
/// If this returns `None`, the entry should be removed from the orphan list.
fn valid_orphan<'gc>(
    dobj: DisplayObjectWeak<'gc>,
    mc: &Mutation<'gc>,
) -> Option<DisplayObject<'gc>> {
    dobj.upgrade(mc).filter(|dobj| dobj.parent().is_none())
}
