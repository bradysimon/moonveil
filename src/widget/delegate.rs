//! Macros for widgets that forward everything to a single inner widget.

/// Implements `Meta` by forwarding to the `$inner` field.
macro_rules! meta {
    ($inner:tt) => {
        fn is_void(&self) -> bool {
            iced_core::widget::Meta::is_void(&self.$inner)
        }
    };
}

/// Implements `Widget` by forwarding to the `$inner` field.
macro_rules! widget {
    ($inner:tt) => {
        fn tag(&self) -> iced_core::widget::tree::Tag {
            iced_core::Widget::<Message, crate::Theme, Renderer>::tag(&self.$inner)
        }

        fn state(&self) -> iced_core::widget::tree::State {
            iced_core::Widget::<Message, crate::Theme, Renderer>::state(&self.$inner)
        }

        fn diff(&mut self, tree: &mut iced_core::widget::Tree) {
            iced_core::Widget::<Message, crate::Theme, Renderer>::diff(&mut self.$inner, tree);
        }

        fn size(&self) -> iced_core::Size<iced_core::Length> {
            iced_core::Widget::<Message, crate::Theme, Renderer>::size(&self.$inner)
        }

        fn layout(
            &mut self,
            tree: &mut iced_core::widget::Tree,
            renderer: &Renderer,
            limits: &iced_core::layout::Limits,
        ) {
            iced_core::Widget::<Message, crate::Theme, Renderer>::layout(
                &mut self.$inner,
                tree,
                renderer,
                limits,
            );
        }

        fn operate(
            &mut self,
            tree: &mut iced_core::widget::Tree,
            layout: iced_core::Layout,
            viewport: &iced_core::Rectangle,
            renderer: &Renderer,
            operation: &mut dyn iced_core::widget::Operation,
        ) {
            iced_core::Widget::<Message, crate::Theme, Renderer>::operate(
                &mut self.$inner,
                tree,
                layout,
                viewport,
                renderer,
                operation,
            );
        }

        fn update(
            &mut self,
            tree: &mut iced_core::widget::Tree,
            event: &iced_core::Event,
            layout: iced_core::Layout,
            cursor: iced_core::mouse::Cursor,
            renderer: &Renderer,
            shell: &mut iced_core::Shell<'_, Message>,
            viewport: &iced_core::Rectangle,
        ) {
            iced_core::Widget::<Message, crate::Theme, Renderer>::update(
                &mut self.$inner,
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
            tree: &iced_core::widget::Tree,
            layout: iced_core::Layout,
            cursor: iced_core::mouse::Cursor,
            viewport: &iced_core::Rectangle,
            renderer: &Renderer,
        ) -> iced_core::mouse::Interaction {
            iced_core::Widget::<Message, crate::Theme, Renderer>::mouse_interaction(
                &self.$inner,
                tree,
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn draw(
            &self,
            tree: &iced_core::widget::Tree,
            renderer: &mut Renderer,
            theme: &crate::Theme,
            style: &iced_core::renderer::Style,
            layout: iced_core::Layout,
            cursor: iced_core::mouse::Cursor,
            viewport: &iced_core::Rectangle,
        ) {
            iced_core::Widget::<Message, crate::Theme, Renderer>::draw(
                &self.$inner,
                tree,
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut iced_core::widget::Tree,
            layout: iced_core::Layout,
            renderer: &Renderer,
            viewport: &iced_core::Rectangle,
            translation: iced_core::Vector,
            window: iced_core::Size,
        ) -> Vec<iced_core::overlay::Element<'b, Message, crate::Theme, Renderer>> {
            iced_core::Widget::<Message, crate::Theme, Renderer>::overlay(
                &mut self.$inner,
                tree,
                layout,
                renderer,
                viewport,
                translation,
                window,
            )
        }
    };
}

pub(super) use {meta, widget};
