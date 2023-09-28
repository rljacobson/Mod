# Design Decisions for Object-Oriented Rust

## Multiple different "things" that share functionality

### trait with `members`/`members_mut` methods

Pros:
 * Can have shared data
 * Can have shared implementation
 * Shared implementation can access shared data
 * Single function can take any parameter of type `&dyn TraitName`.
 * Functions on the trait can be overridden in the trait impl.
 * Extending to new implementors is not a breaking change–-except for code using `downcast` under assumption of 
   exhaustion

Cons: 

 * Non-trait functions that dispatch on different trait impls must use `thing.as_any().downcast::<ThingImpl>()`
 * No encapsulation of shared data
 * No encapsulation of shared implementation
 * Specific implementing type is hidden (need to use `downcast`)
 * Implementors are specified at implementor declaration rather than location of trait
 * Cannot infallibly assume exhaustion in code branching on concrete implementations
 * Must be accessed from behind a reference
 * Must meet trait object restrictions

### Regular enum (variants have common data members)

Pros:
 * Can have shared data
 * Can have shared implementation
 * Shared implementation can access shared data
 * Single function can take any parameter of type `ThingEnum`.
 * Easy for non-enum functions to dispatch on different variants
 * Dispatch on variants is checked for exhaustion
 * No need to worry about restrictions on "trait objects"
 * Does not need to be behind a reference
 

Cons:
 * Non-shared implementation is impossible
 * Common data members need to be destructured each time, or else need accessors that destructure
 * Shared implementation cannot be overridden
 * Shared data members must be repeated for every variant
 * No encapsulation of shared data
 * No encapsulation of shared implementation
 * Extending to new implementors _is_ a breaking change
 * Variants are not _types_ in the usual sense.


### enum with `members` data member in each variant

One of the primary downsides to regular enum is common data members need to be destructured each time they are accessed.
This can be solved the same was it is solved with traits: either with data accessors (getters/setters) or with a single
accessor and a shared `member` member. This also solves the awkwardness of repeating the definition of shared data
members for every variant.

The second primary downside is with non-shared implementation. This can be implemented on a special type which
non-shared data is encapsulated by. This provides encapsulation of both non-shared data members and nonshared
implementation. The cost is special access syntax for variant-specific data members and methods.

### struct with enum 

```rust
pub enum PreEquationKind {
   Equation {
      widget: usize,
   },
   
   Rule {
      gadget: i32
   }
}

pub struct PreEquation {
  name: IString,
  kind: PreEq::Equation
}
```


## Example use cases

### Access shared data
```rust
impl Gadget {
   // If Widget were a trait
   pub fn gadginate_widget(&mut self, widget: &dyn Widget) {
      // Access private Gadget member
     self.widget_gadgination_count += 1;
     self.wingnuts.push(widget.wingnut());
   }

   // If Widget were an enum
   pub fn gadginate_widget_enum(&mut self, widget: &WidgetEnum) {
      // Access private Gadget member
      self.widget_gadgination_count += 1;
      match widget {
         Widget::Siberian{wingnut, ..}
         | Widget::Mongolian{wingnut, ..}
         | Widget::Serbian{wingnut, ..}
         | Widget::Nubian{wingnut, ..}
         | Widget::Antarctic{wingnut, ..} => {
            self.wingnuts.push(widget);
         }
      }
      
      // … or use accessors just like trait impl.
      self.wingnuts.push(widget.wingnut());
   }
}
```

### Dynamic dispatch of method

If Widget were a trait
```rust
impl Gadget {
  pub fn gadginate_widget(&mut self, widget: &dyn Widget) {
    // Access private Gadget member
    self.widget_gadgination_count += 1;
    widget.gadginate(self.id); // Different implementations are transparent to calling code
  }
}
impl Widget for SiberianWidget {
   fn gadginate(&mut self, gadget_id: usize) {
      // Custom implementation
      self.siberian_gadgination(gadget_id);
   }
}

// etc.

```

If Widget were an enum
```rust
impl Gadget {
  pub fn gadginate_widget(&mut self, widget: &dyn Widget) {
    // Access private Gadget member
    self.widget_gadgination_count += 1;
    widget.gadginate(self.id); // Different implementations are transparent to calling code
  }
}
impl Widget{
   fn gadginate(&mut self, gadget_id: usize) {
      match widget {
         Widget::Siberian{gadgination_id, ..} => {
            // SiberianWidget implementation
         }
         Widget::Mongolian{other_member, ..} => {
            // MongolianWidget implementation
         }
         Widget::Serbian{something_else} => {
            // SerbianWidget implementation
         }
         Widget::Nubian{wingnut, ..} => {
            // NubianWidget implementation
         }
         Widget::Antarctic{wingnut, ..} => {
            // AntarcticWidget implementation
         }
      }
   }
}

```
