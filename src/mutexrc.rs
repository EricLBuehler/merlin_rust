//! Single-threaded reference-counting pointers. 'Mrc' stands for 'Reference
//! Counted'.
//!
//! The type [`Mrc<T>`][`Mrc`] provides shared ownership of a value of type `T`,
//! allocated in the heap. Invoking [`clone`][clone] on [`Mrc`] produces a new
//! pointer to the same allocation in the heap. When the last [`Mrc`] pointer to a
//! given allocation is destroyed, the value stored in that allocation (often
//! referred to as "inner value") is also dropped.
//!
//! Shared references in Rust disallow mutation by default, and [`Mrc`]
//! is no exception: you cannot generally obtain a mutable reference to
//! something inside an [`Mrc`]. If you need mutability, put a [`Cell`]
//! or [`RefCell`] inside the [`Mrc`]; see [an example of mutability
//! inside an `Mrc`][mutability].
//!
//! [`Mrc`] uses non-atomic reference counting. This means that overhead is very
//! low, but an [`Mrc`] cannot be sent between threads, and consequently [`Mrc`]
//! does not implement [`Send`]. As a result, the Rust compiler
//! will check *at compile time* that you are not sending [`Mrc`]s between
//! threads. If you need multi-threaded, atomic reference counting, use
//! [`sync::Arc`][arc].
//!
//! The [`downgrade`][downgrade] method can be used to create a non-owning
//! [`Weak`] pointer. A [`Weak`] pointer can be [`upgrade`][upgrade]d
//! to an [`Mrc`], but this will return [`None`] if the value stored in the allocation has
//! already been dropped. In other words, `Weak` pointers do not keep the value
//! inside the allocation alive; however, they *do* keep the allocation
//! (the backing store for the inner value) alive.
//!
//! A cycle between [`Mrc`] pointers will never be deallocated. For this reason,
//! [`Weak`] is used to break cycles. For example, a tree could have strong
//! [`Mrc`] pointers from parent nodes to children, and [`Weak`] pointers from
//! children back to their parents.
//!
//! `Mrc<T>` automatically dereferences to `T` (via the [`Deref`] trait),
//! so you can call `T`'s methods on a value of type [`Mrc<T>`][`Mrc`]. To avoid name
//! clashes with `T`'s methods, the methods of [`Mrc<T>`][`Mrc`] itself are associated
//! functions, called using [fully qualified syntax]:
//!
//! ```
//! use std::rc::Mrc;
//!
//! let my_rc = Mrc::new(());
//! let my_weak = Mrc::downgrade(&my_rc);
//! ```
//!
//! `Mrc<T>`'s implementations of traits like `Clone` may also be called using
//! fully qualified syntax. Some people prefer to use fully qualified syntax,
//! while others prefer using method-call syntax.
//!
//! ```
//! use std::rc::Mrc;
//!
//! let rc = Mrc::new(());
//! // Method-call syntax
//! let rc2 = rc.clone();
//! // Fully qualified syntax
//! let rc3 = Mrc::clone(&rc);
//! ```
//!
//! [`Weak<T>`][`Weak`] does not auto-dereference to `T`, because the inner value may have
//! already been dropped.
//!
//! # Cloning references
//!
//! Creating a new reference to the same allocation as an existing reference counted pointer
//! is done using the `Clone` trait implemented for [`Mrc<T>`][`Mrc`] and [`Weak<T>`][`Weak`].
//!
//! ```
//! use std::rc::Mrc;
//!
//! let foo = Mrc::new(vec![1.0, 2.0, 3.0]);
//! // The two syntaxes below are equivalent.
//! let a = foo.clone();
//! let b = Mrc::clone(&foo);
//! // a and b both point to the same memory location as foo.
//! ```
//!
//! The `Mrc::clone(&from)` syntax is the most idiomatic because it conveys more explicitly
//! the meaning of the code. In the example above, this syntax makes it easier to see that
//! this code is creating a new reference rather than copying the whole content of foo.
//!
//! # Examples
//!
//! Consider a scenario where a set of `Gadget`s are owned by a given `Owner`.
//! We want to have our `Gadget`s point to their `Owner`. We can't do this with
//! unique ownership, because more than one gadget may belong to the same
//! `Owner`. [`Mrc`] allows us to share an `Owner` between multiple `Gadget`s,
//! and have the `Owner` remain allocated as long as any `Gadget` points at it.
//!
//! ```
//! use std::rc::Mrc;
//!
//! struct Owner {
//!     name: String,
//!     // ...other fields
//! }
//!
//! struct Gadget {
//!     id: i32,
//!     owner: Mrc<Owner>,
//!     // ...other fields
//! }
//!
//! fn main() {
//!     // Create a reference-counted `Owner`.
//!     let gadget_owner: Mrc<Owner> = Mrc::new(
//!         Owner {
//!             name: "Gadget Man".to_string(),
//!         }
//!     );
//!
//!     // Create `Gadget`s belonging to `gadget_owner`. Cloning the `Mrc<Owner>`
//!     // gives us a new pointer to the same `Owner` allocation, incrementing
//!     // the reference count in the process.
//!     let gadget1 = Gadget {
//!         id: 1,
//!         owner: Mrc::clone(&gadget_owner),
//!     };
//!     let gadget2 = Gadget {
//!         id: 2,
//!         owner: Mrc::clone(&gadget_owner),
//!     };
//!
//!     // Dispose of our local variable `gadget_owner`.
//!     drop(gadget_owner);
//!
//!     // Despite dropping `gadget_owner`, we're still able to print out the name
//!     // of the `Owner` of the `Gadget`s. This is because we've only dropped a
//!     // single `Mrc<Owner>`, not the `Owner` it points to. As long as there are
//!     // other `Mrc<Owner>` pointing at the same `Owner` allocation, it will remain
//!     // live. The field projection `gadget1.owner.name` works because
//!     // `Mrc<Owner>` automatically dereferences to `Owner`.
//!     println!("Gadget {} owned by {}", gadget1.id, gadget1.owner.name);
//!     println!("Gadget {} owned by {}", gadget2.id, gadget2.owner.name);
//!
//!     // At the end of the function, `gadget1` and `gadget2` are destroyed, and
//!     // with them the last counted references to our `Owner`. Gadget Man now
//!     // gets destroyed as well.
//! }
//! ```
//!
//! If our requirements change, and we also need to be able to traverse from
//! `Owner` to `Gadget`, we will run into problems. An [`Mrc`] pointer from `Owner`
//! to `Gadget` introduces a cycle. This means that their
//! reference counts can never reach 0, and the allocation will never be destroyed:
//! a memory leak. In order to get around this, we can use [`Weak`]
//! pointers.
//!
//! Rust actually makes it somewhat difficult to produce this loop in the first
//! place. In order to end up with two values that point at each other, one of
//! them needs to be mutable. This is difficult because [`Mrc`] enforces
//! memory safety by only giving out shared references to the value it wraps,
//! and these don't allow direct mutation. We need to wrap the part of the
//! value we wish to mutate in a [`RefCell`], which provides *interior
//! mutability*: a method to achieve mutability through a shared reference.
//! [`RefCell`] enforces Rust's borrowing rules at runtime.
//!
//! ```
//! use std::rc::Mrc;
//! use std::rc::Weak;
//! use std::cell::RefCell;
//!
//! struct Owner {
//!     name: String,
//!     gadgets: RefCell<Vec<Weak<Gadget>>>,
//!     // ...other fields
//! }
//!
//! struct Gadget {
//!     id: i32,
//!     owner: Mrc<Owner>,
//!     // ...other fields
//! }
//!
//! fn main() {
//!     // Create a reference-counted `Owner`. Note that we've put the `Owner`'s
//!     // vector of `Gadget`s inside a `RefCell` so that we can mutate it through
//!     // a shared reference.
//!     let gadget_owner: Mrc<Owner> = Mrc::new(
//!         Owner {
//!             name: "Gadget Man".to_string(),
//!             gadgets: RefCell::new(vec![]),
//!         }
//!     );
//!
//!     // Create `Gadget`s belonging to `gadget_owner`, as before.
//!     let gadget1 = Mrc::new(
//!         Gadget {
//!             id: 1,
//!             owner: Mrc::clone(&gadget_owner),
//!         }
//!     );
//!     let gadget2 = Mrc::new(
//!         Gadget {
//!             id: 2,
//!             owner: Mrc::clone(&gadget_owner),
//!         }
//!     );
//!
//!     // Add the `Gadget`s to their `Owner`.
//!     {
//!         let mut gadgets = gadget_owner.gadgets.borrow_mut();
//!         gadgets.push(Mrc::downgrade(&gadget1));
//!         gadgets.push(Mrc::downgrade(&gadget2));
//!
//!         // `RefCell` dynamic borrow ends here.
//!     }
//!
//!     // Iterate over our `Gadget`s, printing their details out.
//!     for gadget_weak in gadget_owner.gadgets.borrow().iter() {
//!
//!         // `gadget_weak` is a `Weak<Gadget>`. Since `Weak` pointers can't
//!         // guarantee the allocation still exists, we need to call
//!         // `upgrade`, which returns an `Option<Mrc<Gadget>>`.
//!         //
//!         // In this case we know the allocation still exists, so we simply
//!         // `unwrap` the `Option`. In a more complicated program, you might
//!         // need graceful error handling for a `None` result.
//!
//!         let gadget = gadget_weak.upgrade().unwrap();
//!         println!("Gadget {} owned by {}", gadget.id, gadget.owner.name);
//!     }
//!
//!     // At the end of the function, `gadget_owner`, `gadget1`, and `gadget2`
//!     // are destroyed. There are now no strong (`Mrc`) pointers to the
//!     // gadgets, so they are destroyed. This zeroes the reference count on
//!     // Gadget Man, so he gets destroyed as well.
//! }
//! ```
//!
//! [clone]: Clone::clone
//! [`Cell`]: core::cell::Cell
//! [`RefCell`]: core::cell::RefCell
//! [arc]: crate::sync::Arc
//! [`Deref`]: core::ops::Deref
//! [downgrade]: Mrc::downgrade
//! [upgrade]: Weak::upgrade
//! [mutability]: core::cell#introducing-mutability-inside-of-something-immutable
//! [fully qualified syntax]: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#fully-qualified-syntax-for-disambiguation-calling-methods-with-the-same-name

#[feature(allocator_api)]
#[feature(core_intrinsics)]
#[feature(unsize)]
#[feature(layout_for_ptr)]
#[feature(coerce_unsized)]
#[feature(dispatch_from_dyn)]
#[feature(receiver_trait)]
#[feature(negative_impls)]
#[feature(min_specialization)]
#[feature(dropck_eyepatch)]
#[feature(rustc_attrs)]
#[feature(trusted_len)]
#[feature(pointer_byte_offsets)]
#[feature(slice_ptr_get)]
#[feature(set_ptr_value)]
#[feature(ptr_internals)]
#[feature(strict_provenance)]
#[feature(alloc_layout_extra)]
#[cfg(not(test))]
use std::boxed::Box;
#[cfg(test)]
use std::boxed::Box;

use core::any::Any;
use core::borrow;
use core::cell::Cell;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::intrinsics::abort;
#[cfg(not(no_global_oom_handling))]
use core::iter;
use core::marker::{PhantomData, Unsize};
#[cfg(not(no_global_oom_handling))]
use core::mem::size_of_val;
use core::mem::{self, align_of_val_raw, forget};
use core::ops::{CoerceUnsized, Deref, DispatchFromDyn, Receiver};
use core::panic::{RefUnwindSafe, UnwindSafe};
#[cfg(not(no_global_oom_handling))]
use core::pin::Pin;
use core::ptr::{self, NonNull};
#[cfg(not(no_global_oom_handling))]
use core::slice::from_raw_parts_mut;

#[cfg(not(no_global_oom_handling))]
use std::alloc::handle_alloc_error;
#[cfg(not(no_global_oom_handling))]
use std::alloc::{AllocError, Allocator, Global, Layout};
use std::borrow::{Cow, ToOwned};
#[cfg(not(no_global_oom_handling))]
use std::string::String;
use std::sync::Mutex;
#[cfg(not(no_global_oom_handling))]
use std::vec::Vec;

//COPIED FROM std::alloc 1.70.0
use std::mem::align_of_val;
use std::ptr::Unique;
unsafe fn box_free<T: ?Sized, A: Allocator>(ptr: Unique<T>, alloc: A) {
    unsafe {
        let size = size_of_val(ptr.as_ref());
        let align = align_of_val(ptr.as_ref());
        let layout = Layout::from_size_align_unchecked(size, align);
        alloc.deallocate(From::from(ptr.cast()), layout)
    }
}
/// Specialize clones into pre-allocated, uninitialized memory.
/// Used by `Box::clone` and `Rc`/`Arc::make_mut`.
pub(crate) trait WriteCloneIntoRaw: Sized {
    unsafe fn write_clone_into_raw(&self, target: *mut Self);
}

impl<T: Clone> WriteCloneIntoRaw for T {
    #[inline]
    default unsafe fn write_clone_into_raw(&self, target: *mut Self) {
        // Having allocated *first* may allow the optimizer to create
        // the cloned value in-place, skipping the local and move.
        unsafe { target.write(self.clone()) };
    }
}

impl<T: Copy> WriteCloneIntoRaw for T {
    #[inline]
    unsafe fn write_clone_into_raw(&self, target: *mut Self) {
        // We can always copy in-place, without ever involving a local value.
        unsafe { target.copy_from_nonoverlapping(self, 1) };
    }
}

// This is repr(C) to future-proof against possible field-reordering, which
// would interfere with otherwise safe [into|from]_raw() of transmutable
// inner types.
#[repr(C)]
struct MrcBox<T: ?Sized> {
    strong: Mutex<Cell<usize>>,
    weak: Mutex<Cell<usize>>,
    value: T,
}

/// Calculate layout for `MrcBox<T>` using the inner value's layout
fn rcbox_layout_for_value_layout(layout: Layout) -> Layout {
    // Calculate layout using the given value layout.
    // Previously, layout was calculated on the expression
    // `&*(ptr as *const MrcBox<T>)`, but this created a misaligned
    // reference (see #54908).
    Layout::new::<MrcBox<()>>()
        .extend(layout)
        .unwrap()
        .0
        .pad_to_align()
}

/// A single-threaded reference-counting pointer. 'Mrc' stands for 'Mutex Reference
/// Counted'.
///
/// See the [module-level documentation](./index.html) for more details.
///
/// The inherent methods of `Mrc` are all associated functions, which means
/// that you have to call them as e.g., [`Mrc::get_mut(&mut value)`][get_mut] instead of
/// `value.get_mut()`. This avoids conflicts with methods of the inner type `T`.
///
/// [get_mut]: Mrc::get_mut
pub struct Mrc<T: ?Sized> {
    ptr: NonNull<MrcBox<T>>,
    phantom: PhantomData<MrcBox<T>>,
}

impl<T: ?Sized> !Send for Mrc<T> {}

// Note that this negative impl isn't strictly necessary for correctness,
// as `Mrc` transitively contains a `Cell`, which is itself `!Sync`.
// However, given how important `Mrc`'s `!Sync`-ness is,
// having an explicit negative impl is nice for documentation purposes
// and results in nicer error messages.
impl<T: ?Sized> !Sync for Mrc<T> {}

impl<T: RefUnwindSafe + ?Sized> UnwindSafe for Mrc<T> {}
impl<T: RefUnwindSafe + ?Sized> RefUnwindSafe for Mrc<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Mrc<U>> for Mrc<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Mrc<U>> for Mrc<T> {}

impl<T: ?Sized> Mrc<T> {
    #[inline(always)]
    fn inner(&self) -> &MrcBox<T> {
        // This unsafety is ok because while this Mrc is alive we're guaranteed
        // that the inner pointer is valid.
        unsafe { self.ptr.as_ref() }
    }

    unsafe fn from_inner(ptr: NonNull<MrcBox<T>>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    unsafe fn from_ptr(ptr: *mut MrcBox<T>) -> Self {
        unsafe { Self::from_inner(NonNull::new_unchecked(ptr)) }
    }
}

impl<T> Mrc<T> {
    /// Constructs a new `Mrc<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    /// ```
    #[cfg(not(no_global_oom_handling))]
    pub fn new(value: T) -> Mrc<T> {
        // There is an implicit weak pointer owned by all the strong
        // pointers, which ensures that the weak destructor never frees
        // the allocation while the strong destructor is running, even
        // if the weak pointer is stored inside the strong one.
        unsafe {
            Self::from_inner(
                Box::leak(Box::new(MrcBox {
                    strong: Mutex::new(Cell::new(1)),
                    weak: Mutex::new(Cell::new(1)),
                    value,
                }))
                .into(),
            )
        }
    }

    /// Constructs a new `Mrc<T>` while giving you a `Weak<T>` to the allocation,
    /// to allow you to construct a `T` which holds a weak pointer to itself.
    ///
    /// Generally, a structure circularly referencing itself, either directly or
    /// indirectly, should not hold a strong reference to itself to prevent a memory leak.
    /// Using this function, you get access to the weak pointer during the
    /// initialization of `T`, before the `Mrc<T>` is created, such that you can
    /// clone and store it inside the `T`.
    ///
    /// `new_cyclic` first allocates the managed allocation for the `Mrc<T>`,
    /// then calls your closure, giving it a `Weak<T>` to this allocation,
    /// and only afterwards completes the construction of the `Mrc<T>` by placing
    /// the `T` returned from your closure into the allocation.
    ///
    /// Since the new `Mrc<T>` is not fully-constructed until `Mrc<T>::new_cyclic`
    /// returns, calling [`upgrade`] on the weak reference inside your closure will
    /// fail and result in a `None` value.
    ///
    /// # Panics
    ///
    /// If `data_fn` panics, the panic is propagated to the caller, and the
    /// temporary [`Weak<T>`] is dropped normally.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(dead_code)]
    /// use std::rc::{Mrc, Weak};
    ///
    /// struct Gadget {
    ///     me: Weak<Gadget>,
    /// }
    ///
    /// impl Gadget {
    ///     /// Construct a reference counted Gadget.
    ///     fn new() -> Mrc<Self> {
    ///         // `me` is a `Weak<Gadget>` pointing at the new allocation of the
    ///         // `Mrc` we're constructing.
    ///         Mrc::new_cyclic(|me| {
    ///             // Create the actual struct here.
    ///             Gadget { me: me.clone() }
    ///         })
    ///     }
    ///
    ///     /// Return a reference counted pointer to Self.
    ///     fn me(&self) -> Mrc<Self> {
    ///         self.me.upgrade().unwrap()
    ///     }
    /// }
    /// ```
    /// [`upgrade`]: Weak::upgrade
    #[cfg(not(no_global_oom_handling))]
    pub fn new_cyclic<F>(data_fn: F) -> Mrc<T>
    where
        F: FnOnce(&Weak<T>) -> T,
    {
        // Construct the inner in the "uninitialized" state with a single
        // weak reference.
        let uninit_ptr: NonNull<_> = Box::leak(Box::new(MrcBox {
            strong: Mutex::new(Cell::new(0)),
            weak: Mutex::new(Cell::new(1)),
            value: mem::MaybeUninit::<T>::uninit(),
        }))
        .into();

        let init_ptr: NonNull<MrcBox<T>> = uninit_ptr.cast();

        let weak = Weak { ptr: init_ptr };

        // It's important we don't give up ownership of the weak pointer, or
        // else the memory might be freed by the time `data_fn` returns. If
        // we really wanted to pass ownership, we could create an additional
        // weak pointer for ourselves, but this would result in additional
        // updates to the weak reference count which might not be necessary
        // otherwise.
        let data = data_fn(&weak);

        let strong = unsafe {
            let inner = init_ptr.as_ptr();
            ptr::write(ptr::addr_of_mut!((*inner).value), data);

            let prev_value = (*inner).strong.lock().unwrap().get();
            debug_assert_eq!(prev_value, 0, "No prior strong references should exist");
            (*inner).strong.lock().unwrap().set(1);

            Mrc::from_inner(init_ptr)
        };

        // Strong references should collectively own a shared weak reference,
        // so don't run the destructor for our old weak reference.
        mem::forget(weak);
        strong
    }

    /// Constructs a new `Mrc` with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let mut five = Mrc::<u32>::new_uninit();
    ///
    /// // Deferred initialization:
    /// Mrc::get_mut(&mut five).unwrap().write(5);
    ///
    /// let five = unsafe { five.assume_init() };
    ///
    /// assert_eq!(*five, 5)
    /// ```
    #[cfg(not(no_global_oom_handling))]
    #[must_use]
    pub fn new_uninit() -> Mrc<mem::MaybeUninit<T>> {
        unsafe {
            Mrc::from_ptr(Mrc::allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate(layout),
                |mem| mem as *mut MrcBox<mem::MaybeUninit<T>>,
            ))
        }
    }

    /// Constructs a new `Mrc` with uninitialized contents, with the memory
    /// being filled with `0` bytes.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and
    /// incorrect usage of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let zero = Mrc::<u32>::new_zeroed();
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero, 0)
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    #[cfg(not(no_global_oom_handling))]
    #[must_use]
    pub fn new_zeroed() -> Mrc<mem::MaybeUninit<T>> {
        unsafe {
            Mrc::from_ptr(Mrc::allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate_zeroed(layout),
                |mem| mem as *mut MrcBox<mem::MaybeUninit<T>>,
            ))
        }
    }

    /// Constructs a new `Mrc<T>`, returning an error if the allocation fails
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::try_new(5);
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    pub fn try_new(value: T) -> Result<Mrc<T>, AllocError> {
        // There is an implicit weak pointer owned by all the strong
        // pointers, which ensures that the weak destructor never frees
        // the allocation while the strong destructor is running, even
        // if the weak pointer is stored inside the strong one.
        unsafe {
            Ok(Self::from_inner(
                Box::leak(Box::try_new(MrcBox {
                    strong: Mutex::new(Cell::new(1)),
                    weak: Mutex::new(Cell::new(1)),
                    value,
                })?)
                .into(),
            ))
        }
    }

    /// Constructs a new `Mrc` with uninitialized contents, returning an error if the allocation fails
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api, new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let mut five = Mrc::<u32>::try_new_uninit()?;
    ///
    /// // Deferred initialization:
    /// Mrc::get_mut(&mut five).unwrap().write(5);
    ///
    /// let five = unsafe { five.assume_init() };
    ///
    /// assert_eq!(*five, 5);
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    // #[unstable(feature = "new_uninit", issue = "63291")]
    pub fn try_new_uninit() -> Result<Mrc<mem::MaybeUninit<T>>, AllocError> {
        unsafe {
            Ok(Mrc::from_ptr(Mrc::try_allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate(layout),
                |mem| mem as *mut MrcBox<mem::MaybeUninit<T>>,
            )?))
        }
    }

    /// Constructs a new `Mrc` with uninitialized contents, with the memory
    /// being filled with `0` bytes, returning an error if the allocation fails
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and
    /// incorrect usage of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api, new_uninit)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let zero = Mrc::<u32>::try_new_zeroed()?;
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero, 0);
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    //#[unstable(feature = "new_uninit", issue = "63291")]
    pub fn try_new_zeroed() -> Result<Mrc<mem::MaybeUninit<T>>, AllocError> {
        unsafe {
            Ok(Mrc::from_ptr(Mrc::try_allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate_zeroed(layout),
                |mem| mem as *mut MrcBox<mem::MaybeUninit<T>>,
            )?))
        }
    }
    /// Constructs a new `Pin<Mrc<T>>`. If `T` does not implement `Unpin`, then
    /// `value` will be pinned in memory and unable to be moved.
    #[cfg(not(no_global_oom_handling))]
    #[must_use]
    pub fn pin(value: T) -> Pin<Mrc<T>> {
        unsafe { Pin::new_unchecked(Mrc::new(value)) }
    }

    /// Returns the inner value, if the `Mrc` has exactly one strong reference.
    ///
    /// Otherwise, an [`Err`] is returned with the same `Mrc` that was
    /// passed in.
    ///
    /// This will succeed even if there are outstanding weak references.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let x = Mrc::new(3);
    /// assert_eq!(Mrc::try_unwrap(x), Ok(3));
    ///
    /// let x = Mrc::new(4);
    /// let _y = Mrc::clone(&x);
    /// assert_eq!(*Mrc::try_unwrap(x).unwrap_err(), 4);
    /// ```
    #[inline]
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        if Mrc::strong_count(&this) == 1 {
            unsafe {
                let val = ptr::read(&*this); // copy the contained object

                // Indicate to Weaks that they can't be promoted by decrementing
                // the strong count, and then remove the implicit "strong weak"
                // pointer while also handling drop logic by just crafting a
                // fake Weak.
                this.inner().dec_strong();
                let _weak = Weak { ptr: this.ptr };
                forget(this);
                Ok(val)
            }
        } else {
            Err(this)
        }
    }

    /// Returns the inner value, if the `Mrc` has exactly one strong reference.
    ///
    /// Otherwise, [`None`] is returned and the `Mrc` is dropped.
    ///
    /// This will succeed even if there are outstanding weak references.
    ///
    /// If `Mrc::into_inner` is called on every clone of this `Mrc`,
    /// it is guaranteed that exactly one of the calls returns the inner value.
    /// This means in particular that the inner value is not dropped.
    ///
    /// This is equivalent to `Mrc::try_unwrap(this).ok()`. (Note that these are not equivalent for
    /// [`Arc`](crate::sync::Arc), due to race conditions that do not apply to `Mrc`.)
    #[inline]
    pub fn into_inner(this: Self) -> Option<T> {
        Mrc::try_unwrap(this).ok()
    }
}

impl<T> Mrc<[T]> {
    /// Constructs a new reference-counted slice with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let mut values = Mrc::<[u32]>::new_uninit_slice(3);
    ///
    /// // Deferred initialization:
    /// let data = Mrc::get_mut(&mut values).unwrap();
    /// data[0].write(1);
    /// data[1].write(2);
    /// data[2].write(3);
    ///
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values, [1, 2, 3])
    /// ```
    #[cfg(not(no_global_oom_handling))]
    #[must_use]
    pub fn new_uninit_slice(len: usize) -> Mrc<[mem::MaybeUninit<T>]> {
        unsafe { Mrc::from_ptr(Mrc::allocate_for_slice(len)) }
    }

    /// Constructs a new reference-counted slice with uninitialized contents, with the memory being
    /// filled with `0` bytes.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and
    /// incorrect usage of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let values = Mrc::<[u32]>::new_zeroed_slice(3);
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values, [0, 0, 0])
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    #[cfg(not(no_global_oom_handling))]
    #[must_use]
    pub fn new_zeroed_slice(len: usize) -> Mrc<[mem::MaybeUninit<T>]> {
        unsafe {
            Mrc::from_ptr(Mrc::allocate_for_layout(
                Layout::array::<T>(len).unwrap(),
                |layout| Global.allocate_zeroed(layout),
                |mem| {
                    ptr::slice_from_raw_parts_mut(mem as *mut T, len)
                        as *mut MrcBox<[mem::MaybeUninit<T>]>
                },
            ))
        }
    }
}

impl<T> Mrc<mem::MaybeUninit<T>> {
    /// Converts to `Mrc<T>`.
    ///
    /// # Safety
    ///
    /// As with [`MaybeUninit::assume_init`],
    /// it is up to the caller to guarantee that the inner value
    /// really is in an initialized state.
    /// Calling this when the content is not yet fully initialized
    /// causes immediate undefined behavior.
    ///
    /// [`MaybeUninit::assume_init`]: mem::MaybeUninit::assume_init
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let mut five = Mrc::<u32>::new_uninit();
    ///
    /// // Deferred initialization:
    /// Mrc::get_mut(&mut five).unwrap().write(5);
    ///
    /// let five = unsafe { five.assume_init() };
    ///
    /// assert_eq!(*five, 5)
    /// ```
    #[inline]
    pub unsafe fn assume_init(self) -> Mrc<T> {
        unsafe { Mrc::from_inner(mem::ManuallyDrop::new(self).ptr.cast()) }
    }
}

impl<T> Mrc<[mem::MaybeUninit<T>]> {
    /// Converts to `Mrc<[T]>`.
    ///
    /// # Safety
    ///
    /// As with [`MaybeUninit::assume_init`],
    /// it is up to the caller to guarantee that the inner value
    /// really is in an initialized state.
    /// Calling this when the content is not yet fully initialized
    /// causes immediate undefined behavior.
    ///
    /// [`MaybeUninit::assume_init`]: mem::MaybeUninit::assume_init
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let mut values = Mrc::<[u32]>::new_uninit_slice(3);
    ///
    /// // Deferred initialization:
    /// let data = Mrc::get_mut(&mut values).unwrap();
    /// data[0].write(1);
    /// data[1].write(2);
    /// data[2].write(3);
    ///
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values, [1, 2, 3])
    /// ```
    #[inline]
    pub unsafe fn assume_init(self) -> Mrc<[T]> {
        unsafe { Mrc::from_ptr(mem::ManuallyDrop::new(self).ptr.as_ptr() as _) }
    }
}

impl<T: ?Sized> Mrc<T> {
    /// Consumes the `Mrc`, returning the wrapped pointer.
    ///
    /// To avoid a memory leak the pointer must be converted back to an `Mrc` using
    /// [`Mrc::from_raw`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let x = Mrc::new("hello".to_owned());
    /// let x_ptr = Mrc::into_raw(x);
    /// assert_eq!(unsafe { &*x_ptr }, "hello");
    /// ```
    pub fn into_raw(this: Self) -> *const T {
        let ptr = Self::as_ptr(&this);
        mem::forget(this);
        ptr
    }

    /// Provides a raw pointer to the data.
    ///
    /// The counts are not affected in any way and the `Mrc` is not consumed. The pointer is valid
    /// for as long there are strong counts in the `Mrc`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let x = Mrc::new("hello".to_owned());
    /// let y = Mrc::clone(&x);
    /// let x_ptr = Mrc::as_ptr(&x);
    /// assert_eq!(x_ptr, Mrc::as_ptr(&y));
    /// assert_eq!(unsafe { &*x_ptr }, "hello");
    /// ```
    pub fn as_ptr(this: &Self) -> *const T {
        let ptr: *mut MrcBox<T> = NonNull::as_ptr(this.ptr);

        // SAFETY: This cannot go through Deref::deref or Mrc::inner because
        // this is required to retain raw/mut provenance such that e.g. `get_mut` can
        // write through the pointer after the Mrc is recovered through `from_raw`.
        unsafe { ptr::addr_of_mut!((*ptr).value) }
    }

    /// Constructs an `Mrc<T>` from a raw pointer.
    ///
    /// The raw pointer must have been previously returned by a call to
    /// [`Mrc<U>::into_raw`][into_raw] where `U` must have the same size
    /// and alignment as `T`. This is trivially true if `U` is `T`.
    /// Note that if `U` is not `T` but has the same size and alignment, this is
    /// basically like transmuting references of different types. See
    /// [`mem::transmute`] for more information on what
    /// restrictions apply in this case.
    ///
    /// The user of `from_raw` has to make sure a specific value of `T` is only
    /// dropped once.
    ///
    /// This function is unsafe because improper use may lead to memory unsafety,
    /// even if the returned `Mrc<T>` is never accessed.
    ///
    /// [into_raw]: Mrc::into_raw
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let x = Mrc::new("hello".to_owned());
    /// let x_ptr = Mrc::into_raw(x);
    ///
    /// unsafe {
    ///     // Convert back to an `Mrc` to prevent leak.
    ///     let x = Mrc::from_raw(x_ptr);
    ///     assert_eq!(&*x, "hello");
    ///
    ///     // Further calls to `Mrc::from_raw(x_ptr)` would be memory-unsafe.
    /// }
    ///
    /// // The memory was freed when `x` went out of scope above, so `x_ptr` is now dangling!
    /// ```
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        let offset = unsafe { data_offset(ptr) };

        // Reverse the offset to find the original MrcBox.
        let rc_ptr = unsafe { ptr.byte_sub(offset) as *mut MrcBox<T> };

        unsafe { Self::from_ptr(rc_ptr) }
    }

    /// Creates a new [`Weak`] pointer to this allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// let weak_five = Mrc::downgrade(&five);
    /// ```
    #[must_use = "this returns a new `Weak` pointer, \
                  without modifying the original `Mrc`"]
    pub fn downgrade(this: &Self) -> Weak<T> {
        this.inner().inc_weak();
        // Make sure we do not create a dangling Weak
        debug_assert!(!is_dangling(this.ptr.as_ptr()));
        Weak { ptr: this.ptr }
    }

    /// Gets the number of [`Weak`] pointers to this allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    /// let _weak_five = Mrc::downgrade(&five);
    ///
    /// assert_eq!(1, Mrc::weak_count(&five));
    /// ```
    #[inline]
    pub fn weak_count(this: &Self) -> usize {
        this.inner().weak() - 1
    }

    /// Gets the number of strong (`Mrc`) pointers to this allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    /// let _also_five = Mrc::clone(&five);
    ///
    /// assert_eq!(2, Mrc::strong_count(&five));
    /// ```
    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong()
    }

    /// Increments the strong reference count on the `Mrc<T>` associated with the
    /// provided pointer by one.
    ///
    /// # Safety
    ///
    /// The pointer must have been obtained through `Mrc::into_raw`, and the
    /// associated `Mrc` instance must be valid (i.e. the strong count must be at
    /// least 1) for the duration of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// unsafe {
    ///     let ptr = Mrc::into_raw(five);
    ///     Mrc::increment_strong_count(ptr);
    ///
    ///     let five = Mrc::from_raw(ptr);
    ///     assert_eq!(2, Mrc::strong_count(&five));
    /// }
    /// ```
    #[inline]
    pub unsafe fn increment_strong_count(ptr: *const T) {
        // Retain Mrc, but don't touch refcount by wrapping in ManuallyDrop
        let rc = unsafe { mem::ManuallyDrop::new(Mrc::<T>::from_raw(ptr)) };
        // Now increase refcount, but don't drop new refcount either
        let _rc_clone: mem::ManuallyDrop<_> = rc.clone();
    }

    /// Decrements the strong reference count on the `Mrc<T>` associated with the
    /// provided pointer by one.
    ///
    /// # Safety
    ///
    /// The pointer must have been obtained through `Mrc::into_raw`, and the
    /// associated `Mrc` instance must be valid (i.e. the strong count must be at
    /// least 1) when invoking this method. This method can be used to release
    /// the final `Mrc` and backing storage, but **should not** be called after
    /// the final `Mrc` has been released.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// unsafe {
    ///     let ptr = Mrc::into_raw(five);
    ///     Mrc::increment_strong_count(ptr);
    ///
    ///     let five = Mrc::from_raw(ptr);
    ///     assert_eq!(2, Mrc::strong_count(&five));
    ///     Mrc::decrement_strong_count(ptr);
    ///     assert_eq!(1, Mrc::strong_count(&five));
    /// }
    /// ```
    #[inline]
    pub unsafe fn decrement_strong_count(ptr: *const T) {
        unsafe { drop(Mrc::from_raw(ptr)) };
    }

    /// Returns `true` if there are no other `Mrc` or [`Weak`] pointers to
    /// this allocation.
    #[inline]
    fn is_unique(this: &Self) -> bool {
        Mrc::weak_count(this) == 0 && Mrc::strong_count(this) == 1
    }

    /// Returns a mutable reference into the given `Mrc`, if there are
    /// no other `Mrc` or [`Weak`] pointers to the same allocation.
    ///
    /// Returns [`None`] otherwise, because it is not safe to
    /// mutate a shared value.
    ///
    /// See also [`make_mut`][make_mut], which will [`clone`][clone]
    /// the inner value when there are other `Mrc` pointers.
    ///
    /// [make_mut]: Mrc::make_mut
    /// [clone]: Clone::clone
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let mut x = Mrc::new(3);
    /// *Mrc::get_mut(&mut x).unwrap() = 4;
    /// assert_eq!(*x, 4);
    ///
    /// let _y = Mrc::clone(&x);
    /// assert!(Mrc::get_mut(&mut x).is_none());
    /// ```
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Mrc::is_unique(this) {
            unsafe { Some(Mrc::get_mut_unchecked(this)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference into the given `Mrc`,
    /// without any check.
    ///
    /// See also [`get_mut`], which is safe and does appropriate checks.
    ///
    /// [`get_mut`]: Mrc::get_mut
    ///
    /// # Safety
    ///
    /// If any other `Mrc` or [`Weak`] pointers to the same allocation exist, then
    /// they must not be dereferenced or have active borrows for the duration
    /// of the returned borrow, and their inner type must be exactly the same as the
    /// inner type of this Mrc (including lifetimes). This is trivially the case if no
    /// such pointers exist, for example immediately after `Mrc::new`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let mut x = Mrc::new(String::new());
    /// unsafe {
    ///     Mrc::get_mut_unchecked(&mut x).push_str("foo")
    /// }
    /// assert_eq!(*x, "foo");
    /// ```
    /// Other `Mrc` pointers to the same allocation must be to the same type.
    /// ```no_run
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let x: Mrc<str> = Mrc::from("Hello, world!");
    /// let mut y: Mrc<[u8]> = x.clone().into();
    /// unsafe {
    ///     // this is Undefined Behavior, because x's inner type is str, not [u8]
    ///     Mrc::get_mut_unchecked(&mut y).fill(0xff); // 0xff is invalid in UTF-8
    /// }
    /// println!("{}", &*x); // Invalid UTF-8 in a str
    /// ```
    /// Other `Mrc` pointers to the same allocation must be to the exact same type, including lifetimes.
    /// ```no_run
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::rc::Mrc;
    ///
    /// let x: Mrc<&str> = Mrc::new("Hello, world!");
    /// {
    ///     let s = String::from("Oh, no!");
    ///     let mut y: Mrc<&str> = x.clone().into();
    ///     unsafe {
    ///         // this is Undefined Behavior, because x's inner type
    ///         // is &'long str, not &'short str
    ///         *Mrc::get_mut_unchecked(&mut y) = &s;
    ///     }
    /// }
    /// println!("{}", &*x); // Use-after-free
    /// ```
    #[inline]
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would conflict with accesses to the reference counts (e.g. by `Weak`).
        unsafe { &mut (*this.ptr.as_ptr()).value }
    }

    #[inline]
    /// Returns `true` if the two `Mrc`s point to the same allocation in a vein similar to
    /// [`ptr::eq`]. See [that function][`ptr::eq`] for caveats when comparing `dyn Trait` pointers.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    /// let same_five = Mrc::clone(&five);
    /// let other_five = Mrc::new(5);
    ///
    /// assert!(Mrc::ptr_eq(&five, &same_five));
    /// assert!(!Mrc::ptr_eq(&five, &other_five));
    /// ```
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<T: Clone> Mrc<T> {
    /// Makes a mutable reference into the given `Mrc`.
    ///
    /// If there are other `Mrc` pointers to the same allocation, then `make_mut` will
    /// [`clone`] the inner value to a new allocation to ensure unique ownership.  This is also
    /// referred to as clone-on-write.
    ///
    /// However, if there are no other `Mrc` pointers to this allocation, but some [`Weak`]
    /// pointers, then the [`Weak`] pointers will be disassociated and the inner value will not
    /// be cloned.
    ///
    /// See also [`get_mut`], which will fail rather than cloning the inner value
    /// or disassociating [`Weak`] pointers.
    ///
    /// [`clone`]: Clone::clone
    /// [`get_mut`]: Mrc::get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let mut data = Mrc::new(5);
    ///
    /// *Mrc::make_mut(&mut data) += 1;         // Won't clone anything
    /// let mut other_data = Mrc::clone(&data); // Won't clone inner data
    /// *Mrc::make_mut(&mut data) += 1;         // Clones inner data
    /// *Mrc::make_mut(&mut data) += 1;         // Won't clone anything
    /// *Mrc::make_mut(&mut other_data) *= 2;   // Won't clone anything
    ///
    /// // Now `data` and `other_data` point to different allocations.
    /// assert_eq!(*data, 8);
    /// assert_eq!(*other_data, 12);
    /// ```
    ///
    /// [`Weak`] pointers will be disassociated:
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let mut data = Mrc::new(75);
    /// let weak = Mrc::downgrade(&data);
    ///
    /// assert!(75 == *data);
    /// assert!(75 == *weak.upgrade().unwrap());
    ///
    /// *Mrc::make_mut(&mut data) += 1;
    ///
    /// assert!(76 == *data);
    /// assert!(weak.upgrade().is_none());
    /// ```
    #[cfg(not(no_global_oom_handling))]
    #[inline]
    pub fn make_mut(this: &mut Self) -> &mut T {
        if Mrc::strong_count(this) != 1 {
            // Gotta clone the data, there are other Mrcs.
            // Pre-allocate memory to allow writing the cloned value directly.
            let mut rc = Self::new_uninit();
            unsafe {
                let data = Mrc::get_mut_unchecked(&mut rc);
                (**this).write_clone_into_raw(data.as_mut_ptr());
                *this = rc.assume_init();
            }
        } else if Mrc::weak_count(this) != 0 {
            // Can just steal the data, all that's left is Weaks
            let mut rc = Self::new_uninit();
            unsafe {
                let data = Mrc::get_mut_unchecked(&mut rc);
                data.as_mut_ptr().copy_from_nonoverlapping(&**this, 1);

                this.inner().dec_strong();
                // Remove implicit strong-weak ref (no need to craft a fake
                // Weak here -- we know other Weaks can clean up for us)
                this.inner().dec_weak();
                ptr::write(this, rc.assume_init());
            }
        }
        // This unsafety is ok because we're guaranteed that the pointer
        // returned is the *only* pointer that will ever be returned to T. Our
        // reference count is guaranteed to be 1 at this point, and we required
        // the `Mrc<T>` itself to be `mut`, so we're returning the only possible
        // reference to the allocation.
        unsafe { &mut this.ptr.as_mut().value }
    }

    /// If we have the only reference to `T` then unwrap it. Otherwise, clone `T` and return the
    /// clone.
    ///
    /// Assuming `rc_t` is of type `Mrc<T>`, this function is functionally equivalent to
    /// `(*rc_t).clone()`, but will avoid cloning the inner value where possible.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(arc_unwrap_or_clone)]
    /// # use std::{ptr, rc::Mrc};
    /// let inner = String::from("test");
    /// let ptr = inner.as_ptr();
    ///
    /// let rc = Mrc::new(inner);
    /// let inner = Mrc::unwrap_or_clone(rc);
    /// // The inner value was not cloned
    /// assert!(ptr::eq(ptr, inner.as_ptr()));
    ///
    /// let rc = Mrc::new(inner);
    /// let rc2 = rc.clone();
    /// let inner = Mrc::unwrap_or_clone(rc);
    /// // Because there were 2 references, we had to clone the inner value.
    /// assert!(!ptr::eq(ptr, inner.as_ptr()));
    /// // `rc2` is the last reference, so when we unwrap it we get back
    /// // the original `String`.
    /// let inner = Mrc::unwrap_or_clone(rc2);
    /// assert!(ptr::eq(ptr, inner.as_ptr()));
    /// ```
    #[inline]
    pub fn unwrap_or_clone(this: Self) -> T {
        Mrc::try_unwrap(this).unwrap_or_else(|rc| (*rc).clone())
    }
}

impl Mrc<dyn Any> {
    /// Attempt to downcast the `Mrc<dyn Any>` to a concrete type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::any::Any;
    /// use std::rc::Mrc;
    ///
    /// fn print_if_string(value: Mrc<dyn Any>) {
    ///     if let Ok(string) = value.downcast::<String>() {
    ///         println!("String ({}): {}", string.len(), string);
    ///     }
    /// }
    ///
    /// let my_string = "Hello World".to_string();
    /// print_if_string(Mrc::new(my_string));
    /// print_if_string(Mrc::new(0i8));
    /// ```
    #[inline]
    pub fn downcast<T: Any>(self) -> Result<Mrc<T>, Mrc<dyn Any>> {
        if (*self).is::<T>() {
            unsafe {
                let ptr = self.ptr.cast::<MrcBox<T>>();
                forget(self);
                Ok(Mrc::from_inner(ptr))
            }
        } else {
            Err(self)
        }
    }

    /// Downcasts the `Mrc<dyn Any>` to a concrete type.
    ///
    /// For a safe alternative see [`downcast`].
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(downcast_unchecked)]
    ///
    /// use std::any::Any;
    /// use std::rc::Mrc;
    ///
    /// let x: Mrc<dyn Any> = Mrc::new(1_usize);
    ///
    /// unsafe {
    ///     assert_eq!(*x.downcast_unchecked::<usize>(), 1);
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// The contained value must be of type `T`. Calling this method
    /// with the incorrect type is *undefined behavior*.
    ///
    ///
    /// [`downcast`]: Self::downcast
    #[inline]
    pub unsafe fn downcast_unchecked<T: Any>(self) -> Mrc<T> {
        unsafe {
            let ptr = self.ptr.cast::<MrcBox<T>>();
            mem::forget(self);
            Mrc::from_inner(ptr)
        }
    }
}

impl<T: ?Sized> Mrc<T> {
    /// Allocates an `MrcBox<T>` with sufficient space for
    /// a possibly-unsized inner value where the value has the layout provided.
    ///
    /// The function `mem_to_rcbox` is called with the data pointer
    /// and must return back a (potentially fat)-pointer for the `MrcBox<T>`.
    #[cfg(not(no_global_oom_handling))]
    unsafe fn allocate_for_layout(
        value_layout: Layout,
        allocate: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocError>,
        mem_to_rcbox: impl FnOnce(*mut u8) -> *mut MrcBox<T>,
    ) -> *mut MrcBox<T> {
        let layout = rcbox_layout_for_value_layout(value_layout);
        unsafe {
            Mrc::try_allocate_for_layout(value_layout, allocate, mem_to_rcbox)
                .unwrap_or_else(|_| handle_alloc_error(layout))
        }
    }

    /// Allocates an `MrcBox<T>` with sufficient space for
    /// a possibly-unsized inner value where the value has the layout provided,
    /// returning an error if allocation fails.
    ///
    /// The function `mem_to_rcbox` is called with the data pointer
    /// and must return back a (potentially fat)-pointer for the `MrcBox<T>`.
    #[inline]
    unsafe fn try_allocate_for_layout(
        value_layout: Layout,
        allocate: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocError>,
        mem_to_rcbox: impl FnOnce(*mut u8) -> *mut MrcBox<T>,
    ) -> Result<*mut MrcBox<T>, AllocError> {
        let layout = rcbox_layout_for_value_layout(value_layout);

        // Allocate for the layout.
        let ptr = allocate(layout)?;

        // Initialize the MrcBox
        let inner = mem_to_rcbox(ptr.as_non_null_ptr().as_ptr());
        unsafe {
            debug_assert_eq!(Layout::for_value(&*inner), layout);

            ptr::write(&mut (*inner).strong, Mutex::new(Cell::new(1)));
            ptr::write(&mut (*inner).weak, Mutex::new(Cell::new(1)));
        }

        Ok(inner)
    }

    /// Allocates an `MrcBox<T>` with sufficient space for an unsized inner value
    #[cfg(not(no_global_oom_handling))]
    unsafe fn allocate_for_ptr(ptr: *const T) -> *mut MrcBox<T> {
        // Allocate for the `MrcBox<T>` using the given value.
        unsafe {
            Self::allocate_for_layout(
                Layout::for_value(&*ptr),
                |layout| Global.allocate(layout),
                |mem| mem.with_metadata_of(ptr as *const MrcBox<T>),
            )
        }
    }

    #[cfg(not(no_global_oom_handling))]
    fn from_box(v: Box<T>) -> Mrc<T> {
        unsafe {
            let (box_unique, alloc) = Box::into_unique(v);
            let bptr = box_unique.as_ptr();

            let value_size = size_of_val(&*bptr);
            let ptr = Self::allocate_for_ptr(bptr);

            // Copy value as bytes
            ptr::copy_nonoverlapping(
                bptr as *const T as *const u8,
                &mut (*ptr).value as *mut _ as *mut u8,
                value_size,
            );

            // Free the allocation without dropping its contents
            box_free(box_unique, alloc);

            Self::from_ptr(ptr)
        }
    }
}

impl<T> Mrc<[T]> {
    /// Allocates an `MrcBox<[T]>` with the given length.
    #[cfg(not(no_global_oom_handling))]
    unsafe fn allocate_for_slice(len: usize) -> *mut MrcBox<[T]> {
        unsafe {
            Self::allocate_for_layout(
                Layout::array::<T>(len).unwrap(),
                |layout| Global.allocate(layout),
                |mem| ptr::slice_from_raw_parts_mut(mem as *mut T, len) as *mut MrcBox<[T]>,
            )
        }
    }

    /// Copy elements from slice into newly allocated `Mrc<[T]>`
    ///
    /// Unsafe because the caller must either take ownership or bind `T: Copy`
    #[cfg(not(no_global_oom_handling))]
    unsafe fn copy_from_slice(v: &[T]) -> Mrc<[T]> {
        unsafe {
            let ptr = Self::allocate_for_slice(v.len());
            ptr::copy_nonoverlapping(v.as_ptr(), &mut (*ptr).value as *mut [T] as *mut T, v.len());
            Self::from_ptr(ptr)
        }
    }

    /// Constructs an `Mrc<[T]>` from an iterator known to be of a certain size.
    ///
    /// Behavior is undefined should the size be wrong.
    #[cfg(not(no_global_oom_handling))]
    unsafe fn from_iter_exact(iter: impl Iterator<Item = T>, len: usize) -> Mrc<[T]> {
        // Panic guard while cloning T elements.
        // In the event of a panic, elements that have been written
        // into the new MrcBox will be dropped, then the memory freed.
        struct Guard<T> {
            mem: NonNull<u8>,
            elems: *mut T,
            layout: Layout,
            n_elems: usize,
        }

        impl<T> Drop for Guard<T> {
            fn drop(&mut self) {
                unsafe {
                    let slice = from_raw_parts_mut(self.elems, self.n_elems);
                    ptr::drop_in_place(slice);

                    Global.deallocate(self.mem, self.layout);
                }
            }
        }

        unsafe {
            let ptr = Self::allocate_for_slice(len);

            let mem = ptr as *mut _ as *mut u8;
            let layout = Layout::for_value(&*ptr);

            // Pointer to first element
            let elems = &mut (*ptr).value as *mut [T] as *mut T;

            let mut guard = Guard {
                mem: NonNull::new_unchecked(mem),
                elems,
                layout,
                n_elems: 0,
            };

            for (i, item) in iter.enumerate() {
                ptr::write(elems.add(i), item);
                guard.n_elems += 1;
            }

            // All clear. Forget the guard so it doesn't free the new MrcBox.
            forget(guard);

            Self::from_ptr(ptr)
        }
    }
}

/// Specialization trait used for `From<&[T]>`.
trait MrcFromSlice<T> {
    fn from_slice(slice: &[T]) -> Self;
}

#[cfg(not(no_global_oom_handling))]
impl<T: Clone> MrcFromSlice<T> for Mrc<[T]> {
    #[inline]
    default fn from_slice(v: &[T]) -> Self {
        unsafe { Self::from_iter_exact(v.iter().cloned(), v.len()) }
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T: Copy> MrcFromSlice<T> for Mrc<[T]> {
    #[inline]
    fn from_slice(v: &[T]) -> Self {
        unsafe { Mrc::copy_from_slice(v) }
    }
}

impl<T: ?Sized> Deref for Mrc<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.inner().value
    }
}

impl<T: ?Sized> Receiver for Mrc<T> {}

unsafe impl<#[may_dangle] T: ?Sized> Drop for Mrc<T> {
    /// Drops the `Mrc`.
    ///
    /// This will decrement the strong reference count. If the strong reference
    /// count reaches zero then the only other references (if any) are
    /// [`Weak`], so we `drop` the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo  = Mrc::new(Foo);
    /// let foo2 = Mrc::clone(&foo);
    ///
    /// drop(foo);    // Doesn't print anything
    /// drop(foo2);   // Prints "dropped!"
    /// ```
    fn drop(&mut self) {
        unsafe {
            self.inner().dec_strong();
            if self.inner().strong() == 0 {
                // destroy the contained object
                ptr::drop_in_place(Self::get_mut_unchecked(self));

                // remove the implicit "strong weak" pointer now that we've
                // destroyed the contents.
                self.inner().dec_weak();

                if self.inner().weak() == 0 {
                    Global.deallocate(self.ptr.cast(), Layout::for_value(self.ptr.as_ref()));
                }
            }
        }
    }
}

impl<T: ?Sized> Clone for Mrc<T> {
    /// Makes a clone of the `Mrc` pointer.
    ///
    /// This creates another pointer to the same allocation, increasing the
    /// strong reference count.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// let _ = Mrc::clone(&five);
    /// ```
    #[inline]
    fn clone(&self) -> Mrc<T> {
        unsafe {
            self.inner().inc_strong();
            Self::from_inner(self.ptr)
        }
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T: Default> Default for Mrc<T> {
    /// Creates a new `Mrc<T>`, with the `Default` value for `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let x: Mrc<i32> = Default::default();
    /// assert_eq!(*x, 0);
    /// ```
    #[inline]
    fn default() -> Mrc<T> {
        Mrc::new(Default::default())
    }
}

trait MrcEqIdent<T: ?Sized + PartialEq> {
    fn eq(&self, other: &Mrc<T>) -> bool;
    fn ne(&self, other: &Mrc<T>) -> bool;
}

impl<T: ?Sized + PartialEq> MrcEqIdent<T> for Mrc<T> {
    #[inline]
    default fn eq(&self, other: &Mrc<T>) -> bool {
        **self == **other
    }

    #[inline]
    default fn ne(&self, other: &Mrc<T>) -> bool {
        **self != **other
    }
}

// Hack to allow specializing on `Eq` even though `Eq` has a method.
#[rustc_unsafe_specialization_marker]
pub(crate) trait MarkerEq: PartialEq<Self> {}

impl<T: Eq> MarkerEq for T {}

/// We're doing this specialization here, and not as a more general optimization on `&T`, because it
/// would otherwise add a cost to all equality checks on refs. We assume that `Mrc`s are used to
/// store large values, that are slow to clone, but also heavy to check for equality, causing this
/// cost to pay off more easily. It's also more likely to have two `Mrc` clones, that point to
/// the same value, than two `&T`s.
///
/// We can only do this when `T: Eq` as a `PartialEq` might be deliberately irreflexive.
impl<T: ?Sized + MarkerEq> MrcEqIdent<T> for Mrc<T> {
    #[inline]
    fn eq(&self, other: &Mrc<T>) -> bool {
        Mrc::ptr_eq(self, other) || **self == **other
    }

    #[inline]
    fn ne(&self, other: &Mrc<T>) -> bool {
        !Mrc::ptr_eq(self, other) && **self != **other
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Mrc<T> {
    /// Equality for two `Mrc`s.
    ///
    /// Two `Mrc`s are equal if their inner values are equal, even if they are
    /// stored in different allocation.
    ///
    /// If `T` also implements `Eq` (implying reflexivity of equality),
    /// two `Mrc`s that point to the same allocation are
    /// always equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert!(five == Mrc::new(5));
    /// ```
    #[inline]
    fn eq(&self, other: &Mrc<T>) -> bool {
        MrcEqIdent::eq(self, other)
    }

    /// Inequality for two `Mrc`s.
    ///
    /// Two `Mrc`s are not equal if their inner values are not equal.
    ///
    /// If `T` also implements `Eq` (implying reflexivity of equality),
    /// two `Mrc`s that point to the same allocation are
    /// always equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert!(five != Mrc::new(6));
    /// ```
    #[inline]
    fn ne(&self, other: &Mrc<T>) -> bool {
        MrcEqIdent::ne(self, other)
    }
}

impl<T: ?Sized + Eq> Eq for Mrc<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for Mrc<T> {
    /// Partial comparison for two `Mrc`s.
    ///
    /// The two are compared by calling `partial_cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert_eq!(Some(Ordering::Less), five.partial_cmp(&Mrc::new(6)));
    /// ```
    #[inline(always)]
    fn partial_cmp(&self, other: &Mrc<T>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }

    /// Less-than comparison for two `Mrc`s.
    ///
    /// The two are compared by calling `<` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert!(five < Mrc::new(6));
    /// ```
    #[inline(always)]
    fn lt(&self, other: &Mrc<T>) -> bool {
        **self < **other
    }

    /// 'Less than or equal to' comparison for two `Mrc`s.
    ///
    /// The two are compared by calling `<=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert!(five <= Mrc::new(5));
    /// ```
    #[inline(always)]
    fn le(&self, other: &Mrc<T>) -> bool {
        **self <= **other
    }

    /// Greater-than comparison for two `Mrc`s.
    ///
    /// The two are compared by calling `>` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert!(five > Mrc::new(4));
    /// ```
    #[inline(always)]
    fn gt(&self, other: &Mrc<T>) -> bool {
        **self > **other
    }

    /// 'Greater than or equal to' comparison for two `Mrc`s.
    ///
    /// The two are compared by calling `>=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert!(five >= Mrc::new(5));
    /// ```
    #[inline(always)]
    fn ge(&self, other: &Mrc<T>) -> bool {
        **self >= **other
    }
}

impl<T: ?Sized + Ord> Ord for Mrc<T> {
    /// Comparison for two `Mrc`s.
    ///
    /// The two are compared by calling `cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Mrc::new(5);
    ///
    /// assert_eq!(Ordering::Less, five.cmp(&Mrc::new(6)));
    /// ```
    #[inline]
    fn cmp(&self, other: &Mrc<T>) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: ?Sized + Hash> Hash for Mrc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for Mrc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mrc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> fmt::Pointer for Mrc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T> From<T> for Mrc<T> {
    /// Converts a generic type `T` into an `Mrc<T>`
    ///
    /// The conversion allocates on the heap and moves `t`
    /// from the stack into it.
    ///
    /// # Example
    /// ```rust
    /// # use std::rc::Mrc;
    /// let x = 5;
    /// let rc = Mrc::new(5);
    ///
    /// assert_eq!(Mrc::from(x), rc);
    /// ```
    fn from(t: T) -> Self {
        Mrc::new(t)
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T: Clone> From<&[T]> for Mrc<[T]> {
    /// Allocate a reference-counted slice and fill it by cloning `v`'s items.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::rc::Mrc;
    /// let original: &[i32] = &[1, 2, 3];
    /// let shared: Mrc<[i32]> = Mrc::from(original);
    /// assert_eq!(&[1, 2, 3], &shared[..]);
    /// ```
    #[inline]
    fn from(v: &[T]) -> Mrc<[T]> {
        <Self as MrcFromSlice<T>>::from_slice(v)
    }
}

#[cfg(not(no_global_oom_handling))]
impl From<&str> for Mrc<str> {
    /// Allocate a reference-counted string slice and copy `v` into it.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::rc::Mrc;
    /// let shared: Mrc<str> = Mrc::from("statue");
    /// assert_eq!("statue", &shared[..]);
    /// ```
    #[inline]
    fn from(v: &str) -> Mrc<str> {
        let rc = Mrc::<[u8]>::from(v.as_bytes());
        unsafe { Mrc::from_raw(Mrc::into_raw(rc) as *const str) }
    }
}

#[cfg(not(no_global_oom_handling))]
impl From<String> for Mrc<str> {
    /// Allocate a reference-counted string slice and copy `v` into it.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::rc::Mrc;
    /// let original: String = "statue".to_owned();
    /// let shared: Mrc<str> = Mrc::from(original);
    /// assert_eq!("statue", &shared[..]);
    /// ```
    #[inline]
    fn from(v: String) -> Mrc<str> {
        Mrc::from(&v[..])
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T: ?Sized> From<Box<T>> for Mrc<T> {
    /// Move a boxed object to a new, reference counted, allocation.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::rc::Mrc;
    /// let original: Box<i32> = Box::new(1);
    /// let shared: Mrc<i32> = Mrc::from(original);
    /// assert_eq!(1, *shared);
    /// ```
    #[inline]
    fn from(v: Box<T>) -> Mrc<T> {
        Mrc::from_box(v)
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T> From<Vec<T>> for Mrc<[T]> {
    /// Allocate a reference-counted slice and move `v`'s items into it.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::rc::Mrc;
    /// let original: Box<Vec<i32>> = Box::new(vec![1, 2, 3]);
    /// let shared: Mrc<Vec<i32>> = Mrc::from(original);
    /// assert_eq!(vec![1, 2, 3], *shared);
    /// ```
    #[inline]
    fn from(mut v: Vec<T>) -> Mrc<[T]> {
        unsafe {
            let rc = Mrc::copy_from_slice(&v);
            // Allow the Vec to free its memory, but not destroy its contents
            v.set_len(0);
            rc
        }
    }
}

impl<'a, B> From<Cow<'a, B>> for Mrc<B>
where
    B: ToOwned + ?Sized,
    Mrc<B>: From<&'a B> + From<B::Owned>,
{
    /// Create a reference-counted pointer from
    /// a clone-on-write pointer by copying its content.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::rc::Mrc;
    /// # use std::borrow::Cow;
    /// let cow: Cow<str> = Cow::Borrowed("eggplant");
    /// let shared: Mrc<str> = Mrc::from(cow);
    /// assert_eq!("eggplant", &shared[..]);
    /// ```
    #[inline]
    fn from(cow: Cow<'a, B>) -> Mrc<B> {
        match cow {
            Cow::Borrowed(s) => Mrc::from(s),
            Cow::Owned(s) => Mrc::from(s),
        }
    }
}

impl From<Mrc<str>> for Mrc<[u8]> {
    /// Converts a reference-counted string slice into a byte slice.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::rc::Mrc;
    /// let string: Mrc<str> = Mrc::from("eggplant");
    /// let bytes: Mrc<[u8]> = Mrc::from(string);
    /// assert_eq!("eggplant".as_bytes(), bytes.as_ref());
    /// ```
    #[inline]
    fn from(rc: Mrc<str>) -> Self {
        // SAFETY: `str` has the same layout as `[u8]`.
        unsafe { Mrc::from_raw(Mrc::into_raw(rc) as *const [u8]) }
    }
}

impl<T, const N: usize> TryFrom<Mrc<[T]>> for Mrc<[T; N]> {
    type Error = Mrc<[T]>;

    fn try_from(boxed_slice: Mrc<[T]>) -> Result<Self, Self::Error> {
        if boxed_slice.len() == N {
            Ok(unsafe { Mrc::from_raw(Mrc::into_raw(boxed_slice) as *mut [T; N]) })
        } else {
            Err(boxed_slice)
        }
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T> FromIterator<T> for Mrc<[T]> {
    /// Takes each element in the `Iterator` and collects it into an `Mrc<[T]>`.
    ///
    /// # Performance characteristics
    ///
    /// ## The general case
    ///
    /// In the general case, collecting into `Mrc<[T]>` is done by first
    /// collecting into a `Vec<T>`. That is, when writing the following:
    ///
    /// ```rust
    /// # use std::rc::Mrc;
    /// let evens: Mrc<[u8]> = (0..10).filter(|&x| x % 2 == 0).collect();
    /// # assert_eq!(&*evens, &[0, 2, 4, 6, 8]);
    /// ```
    ///
    /// this behaves as if we wrote:
    ///
    /// ```rust
    /// # use std::rc::Mrc;
    /// let evens: Mrc<[u8]> = (0..10).filter(|&x| x % 2 == 0)
    ///     .collect::<Vec<_>>() // The first set of allocations happens here.
    ///     .into(); // A second allocation for `Mrc<[T]>` happens here.
    /// # assert_eq!(&*evens, &[0, 2, 4, 6, 8]);
    /// ```
    ///
    /// This will allocate as many times as needed for constructing the `Vec<T>`
    /// and then it will allocate once for turning the `Vec<T>` into the `Mrc<[T]>`.
    ///
    /// ## Iterators of known length
    ///
    /// When your `Iterator` implements `TrustedLen` and is of an exact size,
    /// a single allocation will be made for the `Mrc<[T]>`. For example:
    ///
    /// ```rust
    /// # use std::rc::Mrc;
    /// let evens: Mrc<[u8]> = (0..10).collect(); // Just a single allocation happens here.
    /// # assert_eq!(&*evens, &*(0..10).collect::<Vec<_>>());
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ToMrcSlice::to_rc_slice(iter.into_iter())
    }
}

/// Specialization trait used for collecting into `Mrc<[T]>`.
#[cfg(not(no_global_oom_handling))]
trait ToMrcSlice<T>: Iterator<Item = T> + Sized {
    fn to_rc_slice(self) -> Mrc<[T]>;
}

#[cfg(not(no_global_oom_handling))]
impl<T, I: Iterator<Item = T>> ToMrcSlice<T> for I {
    default fn to_rc_slice(self) -> Mrc<[T]> {
        self.collect::<Vec<T>>().into()
    }
}

#[cfg(not(no_global_oom_handling))]
impl<T, I: iter::TrustedLen<Item = T>> ToMrcSlice<T> for I {
    fn to_rc_slice(self) -> Mrc<[T]> {
        // This is the case for a `TrustedLen` iterator.
        let (low, high) = self.size_hint();
        if let Some(high) = high {
            debug_assert_eq!(
                low,
                high,
                "TrustedLen iterator's size hint is not exact: {:?}",
                (low, high)
            );

            unsafe {
                // SAFETY: We need to ensure that the iterator has an exact length and we have.
                Mrc::from_iter_exact(self, low)
            }
        } else {
            // TrustedLen contract guarantees that `upper_bound == None` implies an iterator
            // length exceeding `usize::MAX`.
            // The default implementation would collect into a vec which would panic.
            // Thus we panic here immediately without invoking `Vec` code.
            panic!("capacity overflow");
        }
    }
}

/// `Weak` is a version of [`Mrc`] that holds a non-owning reference to the
/// managed allocation. The allocation is accessed by calling [`upgrade`] on the `Weak`
/// pointer, which returns an <code>[Option]<[Mrc]\<T>></code>.
///
/// Since a `Weak` reference does not count towards ownership, it will not
/// prevent the value stored in the allocation from being dropped, and `Weak` itself makes no
/// guarantees about the value still being present. Thus it may return [`None`]
/// when [`upgrade`]d. Note however that a `Weak` reference *does* prevent the allocation
/// itself (the backing store) from being deallocated.
///
/// A `Weak` pointer is useful for keeping a temporary reference to the allocation
/// managed by [`Mrc`] without preventing its inner value from being dropped. It is also used to
/// prevent circular references between [`Mrc`] pointers, since mutual owning references
/// would never allow either [`Mrc`] to be dropped. For example, a tree could
/// have strong [`Mrc`] pointers from parent nodes to children, and `Weak`
/// pointers from children back to their parents.
///
/// The typical way to obtain a `Weak` pointer is to call [`Mrc::downgrade`].
///
/// [`upgrade`]: Weak::upgrade
pub struct Weak<T: ?Sized> {
    // This is a `NonNull` to allow optimizing the size of this type in enums,
    // but it is not necessarily a valid pointer.
    // `Weak::new` sets this to `usize::MAX` so that it doesn’t need
    // to allocate space on the heap. That's not a value a real pointer
    // will ever have because MrcBox has alignment at least 2.
    // This is only possible when `T: Sized`; unsized `T` never dangle.
    ptr: NonNull<MrcBox<T>>,
}

impl<T: ?Sized> !Send for Weak<T> {}
impl<T: ?Sized> !Sync for Weak<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Weak<U>> for Weak<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Weak<U>> for Weak<T> {}

impl<T> Weak<T> {
    /// Constructs a new `Weak<T>`, without allocating any memory.
    /// Calling [`upgrade`] on the return value always gives [`None`].
    ///
    /// [`upgrade`]: Weak::upgrade
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Weak;
    ///
    /// let empty: Weak<i64> = Weak::new();
    /// assert!(empty.upgrade().is_none());
    /// ```
    #[must_use]
    pub const fn new() -> Weak<T> {
        Weak {
            ptr: unsafe { NonNull::new_unchecked(ptr::invalid_mut::<MrcBox<T>>(usize::MAX)) },
        }
    }
}

pub(crate) fn is_dangling<T: ?Sized>(ptr: *mut T) -> bool {
    (ptr as *mut ()).addr() == usize::MAX
}

/// Helper type to allow accessing the reference counts without
/// making any assertions about the data field.
struct WeakInner {
    weak: Cell<usize>,
    strong: Cell<usize>,
}

impl<T: ?Sized> Weak<T> {
    /// Returns a raw pointer to the object `T` pointed to by this `Weak<T>`.
    ///
    /// The pointer is valid only if there are some strong references. The pointer may be dangling,
    /// unaligned or even [`null`] otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    /// use std::ptr;
    ///
    /// let strong = Mrc::new("hello".to_owned());
    /// let weak = Mrc::downgrade(&strong);
    /// // Both point to the same object
    /// assert!(ptr::eq(&*strong, weak.as_ptr()));
    /// // The strong here keeps it alive, so we can still access the object.
    /// assert_eq!("hello", unsafe { &*weak.as_ptr() });
    ///
    /// drop(strong);
    /// // But not any more. We can do weak.as_ptr(), but accessing the pointer would lead to
    /// // undefined behaviour.
    /// // assert_eq!("hello", unsafe { &*weak.as_ptr() });
    /// ```
    ///
    /// [`null`]: ptr::null
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        let ptr: *mut MrcBox<T> = NonNull::as_ptr(self.ptr);

        if is_dangling(ptr) {
            // If the pointer is dangling, we return the sentinel directly. This cannot be
            // a valid payload address, as the payload is at least as aligned as MrcBox (usize).
            ptr as *const T
        } else {
            // SAFETY: if is_dangling returns false, then the pointer is dereferenceable.
            // The payload may be dropped at this point, and we have to maintain provenance,
            // so use raw pointer manipulation.
            unsafe { ptr::addr_of_mut!((*ptr).value) }
        }
    }

    /// Consumes the `Weak<T>` and turns it into a raw pointer.
    ///
    /// This converts the weak pointer into a raw pointer, while still preserving the ownership of
    /// one weak reference (the weak count is not modified by this operation). It can be turned
    /// back into the `Weak<T>` with [`from_raw`].
    ///
    /// The same restrictions of accessing the target of the pointer as with
    /// [`as_ptr`] apply.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::{Mrc, Weak};
    ///
    /// let strong = Mrc::new("hello".to_owned());
    /// let weak = Mrc::downgrade(&strong);
    /// let raw = weak.into_raw();
    ///
    /// assert_eq!(1, Mrc::weak_count(&strong));
    /// assert_eq!("hello", unsafe { &*raw });
    ///
    /// drop(unsafe { Weak::from_raw(raw) });
    /// assert_eq!(0, Mrc::weak_count(&strong));
    /// ```
    ///
    /// [`from_raw`]: Weak::from_raw
    /// [`as_ptr`]: Weak::as_ptr
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_raw(self) -> *const T {
        let result = self.as_ptr();
        mem::forget(self);
        result
    }

    /// Converts a raw pointer previously created by [`into_raw`] back into `Weak<T>`.
    ///
    /// This can be used to safely get a strong reference (by calling [`upgrade`]
    /// later) or to deallocate the weak count by dropping the `Weak<T>`.
    ///
    /// It takes ownership of one weak reference (with the exception of pointers created by [`new`],
    /// as these don't own anything; the method still works on them).
    ///
    /// # Safety
    ///
    /// The pointer must have originated from the [`into_raw`] and must still own its potential
    /// weak reference.
    ///
    /// It is allowed for the strong count to be 0 at the time of calling this. Nevertheless, this
    /// takes ownership of one weak reference currently represented as a raw pointer (the weak
    /// count is not modified by this operation) and therefore it must be paired with a previous
    /// call to [`into_raw`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::{Mrc, Weak};
    ///
    /// let strong = Mrc::new("hello".to_owned());
    ///
    /// let raw_1 = Mrc::downgrade(&strong).into_raw();
    /// let raw_2 = Mrc::downgrade(&strong).into_raw();
    ///
    /// assert_eq!(2, Mrc::weak_count(&strong));
    ///
    /// assert_eq!("hello", &*unsafe { Weak::from_raw(raw_1) }.upgrade().unwrap());
    /// assert_eq!(1, Mrc::weak_count(&strong));
    ///
    /// drop(strong);
    ///
    /// // Decrement the last weak count.
    /// assert!(unsafe { Weak::from_raw(raw_2) }.upgrade().is_none());
    /// ```
    ///
    /// [`into_raw`]: Weak::into_raw
    /// [`upgrade`]: Weak::upgrade
    /// [`new`]: Weak::new
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        // See Weak::as_ptr for context on how the input pointer is derived.

        let ptr = if is_dangling(ptr as *mut T) {
            // This is a dangling Weak.
            ptr as *mut MrcBox<T>
        } else {
            // Otherwise, we're guaranteed the pointer came from a nondangling Weak.
            // SAFETY: data_offset is safe to call, as ptr references a real (potentially dropped) T.
            let offset = unsafe { data_offset(ptr) };
            // Thus, we reverse the offset to get the whole MrcBox.
            // SAFETY: the pointer originated from a Weak, so this offset is safe.
            unsafe { ptr.byte_sub(offset) as *mut MrcBox<T> }
        };

        // SAFETY: we now have recovered the original Weak pointer, so can create the Weak.
        Weak {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    /// Attempts to upgrade the `Weak` pointer to an [`Mrc`], delaying
    /// dropping of the inner value if successful.
    ///
    /// Returns [`None`] if the inner value has since been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let five = Mrc::new(5);
    ///
    /// let weak_five = Mrc::downgrade(&five);
    ///
    /// let strong_five: Option<Mrc<_>> = weak_five.upgrade();
    /// assert!(strong_five.is_some());
    ///
    /// // Destroy all strong pointers.
    /// drop(strong_five);
    /// drop(five);
    ///
    /// assert!(weak_five.upgrade().is_none());
    /// ```
    #[must_use = "this returns a new `Mrc`, \
                  without modifying the original weak pointer"]
    pub fn upgrade(&self) -> Option<Mrc<T>> {
        let inner = self.inner()?;

        if inner.strong() == 0 {
            None
        } else {
            unsafe {
                inner.inc_strong();
                Some(Mrc::from_inner(self.ptr))
            }
        }
    }

    /// Gets the number of strong (`Mrc`) pointers pointing to this allocation.
    ///
    /// If `self` was created using [`Weak::new`], this will return 0.
    #[must_use]
    pub fn strong_count(&self) -> usize {
        if let Some(inner) = self.inner() {
            inner.strong()
        } else {
            0
        }
    }

    /// Gets the number of `Weak` pointers pointing to this allocation.
    ///
    /// If no strong pointers remain, this will return zero.
    #[must_use]
    pub fn weak_count(&self) -> usize {
        self.inner()
            .map(|inner| {
                if inner.strong() > 0 {
                    inner.weak() - 1 // subtract the implicit weak ptr
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    /// Returns `None` when the pointer is dangling and there is no allocated `MrcBox`,
    /// (i.e., when this `Weak` was created by `Weak::new`).
    #[inline]
    fn inner(&self) -> Option<WeakInner> {
        if is_dangling(self.ptr.as_ptr()) {
            None
        } else {
            // We are careful to *not* create a reference covering the "data" field, as
            // the field may be mutated concurrently (for example, if the last `Mrc`
            // is dropped, the data field will be dropped in-place).
            Some(unsafe {
                let ptr = self.ptr.as_ptr();
                let strong: Cell<usize> = (*ptr).strong.lock().unwrap().deref().clone();
                let weak: Cell<usize> = (*ptr).weak.lock().unwrap().deref().clone();
                WeakInner { strong, weak }
            })
        }
    }

    /// Returns `true` if the two `Weak`s point to the same allocation similar to [`ptr::eq`], or if
    /// both don't point to any allocation (because they were created with `Weak::new()`). See [that
    /// function][`ptr::eq`] for caveats when comparing `dyn Trait` pointers.
    ///
    /// # Notes
    ///
    /// Since this compares pointers it means that `Weak::new()` will equal each
    /// other, even though they don't point to any allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Mrc;
    ///
    /// let first_rc = Mrc::new(5);
    /// let first = Mrc::downgrade(&first_rc);
    /// let second = Mrc::downgrade(&first_rc);
    ///
    /// assert!(first.ptr_eq(&second));
    ///
    /// let third_rc = Mrc::new(5);
    /// let third = Mrc::downgrade(&third_rc);
    ///
    /// assert!(!first.ptr_eq(&third));
    /// ```
    ///
    /// Comparing `Weak::new`.
    ///
    /// ```
    /// use std::rc::{Mrc, Weak};
    ///
    /// let first = Weak::new();
    /// let second = Weak::new();
    /// assert!(first.ptr_eq(&second));
    ///
    /// let third_rc = Mrc::new(());
    /// let third = Mrc::downgrade(&third_rc);
    /// assert!(!first.ptr_eq(&third));
    /// ```
    #[inline]
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

unsafe impl<#[may_dangle] T: ?Sized> Drop for Weak<T> {
    /// Drops the `Weak` pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::{Mrc, Weak};
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo = Mrc::new(Foo);
    /// let weak_foo = Mrc::downgrade(&foo);
    /// let other_weak_foo = Weak::clone(&weak_foo);
    ///
    /// drop(weak_foo);   // Doesn't print anything
    /// drop(foo);        // Prints "dropped!"
    ///
    /// assert!(other_weak_foo.upgrade().is_none());
    /// ```
    fn drop(&mut self) {
        let inner = if let Some(inner) = self.inner() {
            inner
        } else {
            return;
        };

        inner.dec_weak();
        // the weak count starts at 1, and will only go to zero if all
        // the strong pointers have disappeared.
        if inner.weak() == 0 {
            unsafe {
                Global.deallocate(self.ptr.cast(), Layout::for_value_raw(self.ptr.as_ptr()));
            }
        }
    }
}

impl<T: ?Sized> Clone for Weak<T> {
    /// Makes a clone of the `Weak` pointer that points to the same allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::{Mrc, Weak};
    ///
    /// let weak_five = Mrc::downgrade(&Mrc::new(5));
    ///
    /// let _ = Weak::clone(&weak_five);
    /// ```
    #[inline]
    fn clone(&self) -> Weak<T> {
        if let Some(inner) = self.inner() {
            inner.inc_weak()
        }
        Weak { ptr: self.ptr }
    }
}

impl<T: ?Sized> fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Weak)")
    }
}

impl<T> Default for Weak<T> {
    /// Constructs a new `Weak<T>`, without allocating any memory.
    /// Calling [`upgrade`] on the return value always gives [`None`].
    ///
    /// [`upgrade`]: Weak::upgrade
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Weak;
    ///
    /// let empty: Weak<i64> = Default::default();
    /// assert!(empty.upgrade().is_none());
    /// ```
    fn default() -> Weak<T> {
        Weak::new()
    }
}

// NOTE: We checked_add here to deal with mem::forget safely. In particular
// if you mem::forget Mrcs (or Weaks), the ref-count can overflow, and then
// you can free the allocation while outstanding Mrcs (or Weaks) exist.
// We abort because this is such a degenerate scenario that we don't care about
// what happens -- no real program should ever experience this.
//
// This should have negligible overhead since you don't actually need to
// clone these much in Rust thanks to ownership and move-semantics.

#[doc(hidden)]
trait MrcInnerPtr {
    fn weak_ref(&self) -> Cell<usize>;
    fn strong_ref(&self) -> Cell<usize>;

    #[inline]
    fn strong(&self) -> usize {
        self.strong_ref().get()
    }

    #[inline]
    fn inc_strong(&self) {
        let strong = self.strong();

        // We insert an `assume` here to hint LLVM at an otherwise
        // missed optimization.
        // SAFETY: The reference count will never be zero when this is
        // called.
        unsafe {
            core::intrinsics::assume(strong != 0);
        }

        let strong = strong.wrapping_add(1);
        self.strong_ref().set(strong);

        // We want to abort on overflow instead of dropping the value.
        // Checking for overflow after the store instead of before
        // allows for slightly better code generation.
        if core::intrinsics::unlikely(strong == 0) {
            abort();
        }
    }

    #[inline]
    fn dec_strong(&self) {
        self.strong_ref().set(self.strong() - 1);
    }

    #[inline]
    fn weak(&self) -> usize {
        self.weak_ref().get()
    }

    #[inline]
    fn inc_weak(&self) {
        let weak = self.weak();

        // We insert an `assume` here to hint LLVM at an otherwise
        // missed optimization.
        // SAFETY: The reference count will never be zero when this is
        // called.
        unsafe {
            core::intrinsics::assume(weak != 0);
        }

        let weak = weak.wrapping_add(1);
        self.weak_ref().set(weak);

        // We want to abort on overflow instead of dropping the value.
        // Checking for overflow after the store instead of before
        // allows for slightly better code generation.
        if core::intrinsics::unlikely(weak == 0) {
            abort();
        }
    }

    #[inline]
    fn dec_weak(&self) {
        self.weak_ref().set(self.weak() - 1);
    }
}

impl<T: ?Sized> MrcInnerPtr for MrcBox<T> {
    #[inline(always)]
    fn weak_ref(&self) -> Cell<usize> {
        self.weak.lock().unwrap().deref().clone()
    }

    #[inline(always)]
    fn strong_ref(&self) -> Cell<usize> {
        self.strong.lock().unwrap().deref().clone()
    }
}

impl<'a> MrcInnerPtr for WeakInner {
    #[inline(always)]
    fn weak_ref(&self) -> Cell<usize> {
        self.weak.clone()
    }

    #[inline(always)]
    fn strong_ref(&self) -> Cell<usize> {
        self.strong.clone()
    }
}

impl<T: ?Sized> borrow::Borrow<T> for Mrc<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T: ?Sized> AsRef<T> for Mrc<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T: ?Sized> Unpin for Mrc<T> {}

/// Get the offset within an `MrcBox` for the payload behind a pointer.
///
/// # Safety
///
/// The pointer must point to (and have valid metadata for) a previously
/// valid instance of T, but the T is allowed to be dropped.
unsafe fn data_offset<T: ?Sized>(ptr: *const T) -> usize {
    // Align the unsized value to the end of the MrcBox.
    // Because MrcBox is repr(C), it will always be the last field in memory.
    // SAFETY: since the only unsized types possible are slices, trait objects,
    // and extern types, the input safety requirement is currently enough to
    // satisfy the requirements of align_of_val_raw; this is an implementation
    // detail of the language that must not be relied upon outside of std.
    unsafe { data_offset_align(align_of_val_raw(ptr)) }
}

#[inline]
fn data_offset_align(align: usize) -> usize {
    let layout = Layout::new::<MrcBox<()>>();
    layout.size() + layout.padding_needed_for(align)
}
