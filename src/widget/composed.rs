use crate::Theme;
use iced_core::{
    Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget, layout, mouse, overlay,
    renderer,
    widget::{Meta, Operation, Tree, tree},
};
use std::cell::RefCell;

/// Settings that build a widget out of other widgets.
///
/// Wrap them in a [`Composed`] to use them as a widget.
pub trait Compose {
    /// The widget built from these settings.
    type View;

    /// Builds the widget.
    fn view(self) -> Self::View;
}

/// A widget that builds its view from [`Compose`] settings when first used.
///
/// Configure it before handing it to a parent: the first query consumes the settings,
/// and later configuration is ignored (asserted in debug builds).
pub struct Composed<C: Compose>(RefCell<Stage<C>>);

/// The stage of a composed widget in the build lifecycle.
enum Stage<C: Compose> {
    /// The composed widget has not yet built its view.
    Pending(C),
    /// The composed view has been built.
    Built(C::View),
    /// Left behind only if [`Compose::view`] panicked.
    Poisoned,
}

impl<C: Compose> Stage<C> {
    /// Builds the view if it hasn't been built yet.
    fn build(&mut self) {
        if let Self::Pending(_) = self {
            let Self::Pending(settings) = std::mem::replace(self, Self::Poisoned) else {
                unreachable!()
            };
            *self = Self::Built(settings.view());
        }
    }

    /// Returns a reference to the built view, panicking if it hasn't been built yet.
    fn view(&self) -> &C::View {
        let Self::Built(view) = self else {
            panic!("composed widget panicked while building its view");
        };
        view
    }

    /// Returns a mutable reference to the built view, building it if necessary.
    fn view_mut(&mut self) -> &mut C::View {
        self.build();
        let Self::Built(view) = self else {
            panic!("composed widget panicked while building its view");
        };
        view
    }
}

impl<C: Compose> From<C> for Composed<C> {
    fn from(settings: C) -> Self {
        Self(RefCell::new(Stage::Pending(settings)))
    }
}

impl<C: Compose> Composed<C> {
    pub(crate) fn configure(&mut self, f: impl FnOnce(&mut C)) {
        match self.0.get_mut() {
            Stage::Pending(settings) => f(settings),
            Stage::Built(_) | Stage::Poisoned => {
                debug_assert!(false, "composed widget configured after first use");
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn settings_mut(&mut self) -> &mut C {
        let Stage::Pending(settings) = self.0.get_mut() else {
            panic!("composed widget inspected after first use");
        };
        settings
    }

    fn with_view<T>(&self, f: impl FnOnce(&C::View) -> T) -> T {
        if matches!(*self.0.borrow(), Stage::Pending(_)) {
            self.0.borrow_mut().build();
        }
        let stage = self.0.borrow();
        f(stage.view())
    }

    fn view_mut(&mut self) -> &mut C::View {
        self.0.get_mut().view_mut()
    }
}

impl<C> Meta for Composed<C>
where
    C: Compose,
    C::View: Meta,
{
    fn is_void(&self) -> bool {
        self.with_view(Meta::is_void)
    }
}

impl<C, Message, Renderer> Widget<Message, Theme, Renderer> for Composed<C>
where
    C: Compose,
    C::View: Widget<Message, Theme, Renderer>,
{
    fn size(&self) -> Size<Length> {
        self.with_view(Widget::<Message, Theme, Renderer>::size)
    }

    fn tag(&self) -> tree::Tag {
        self.with_view(Widget::<Message, Theme, Renderer>::tag)
    }

    fn state(&self) -> tree::State {
        self.with_view(Widget::<Message, Theme, Renderer>::state)
    }

    fn diff(&mut self, tree: &mut Tree) {
        Widget::<Message, Theme, Renderer>::diff(self.view_mut(), tree);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        Widget::<Message, Theme, Renderer>::layout(self.view_mut(), tree, renderer, limits);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        Widget::<Message, Theme, Renderer>::operate(
            self.view_mut(),
            tree,
            layout,
            viewport,
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        Widget::<Message, Theme, Renderer>::update(
            self.view_mut(),
            tree,
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_view(|view| {
            Widget::<Message, Theme, Renderer>::mouse_interaction(
                view, tree, layout, cursor, viewport, renderer,
            )
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.with_view(|view| {
            Widget::<Message, Theme, Renderer>::draw(
                view, tree, renderer, theme, style, layout, cursor, viewport,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        Widget::<Message, Theme, Renderer>::overlay(
            self.view_mut(),
            tree,
            layout,
            renderer,
            viewport,
            translation,
            window,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::widget::badge::badge;

    #[test]
    #[cfg_attr(
        debug_assertions,
        should_panic(expected = "configured after first use")
    )]
    fn configuring_after_first_use_is_ignored_in_release() {
        let label = badge("New");
        let _ = iced_core::Widget::<(), crate::Theme, ()>::size(&label);
        let _ = label.width(10);
    }
}
