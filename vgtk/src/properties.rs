use std::marker::PhantomData;

use glib::{Cast, GString};
use gtk::{IconSize, Image, ImageExt, Widget};

pub struct PropertyValue<'a, A, Get, Set>
where
    A: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set> + 'a,
{
    value: A,
    lifetime: PhantomData<&'a (Get, Set)>,
}

impl<'a, A, Get, Set> PropertyValue<'a, A, Get, Set>
where
    A: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set> + 'a,
{
    pub fn new(value: A) -> Self {
        Self {
            value,
            lifetime: PhantomData,
        }
    }

    pub fn compare(&self, value: Get) -> bool {
        A::property_compare(value, &self.value)
    }

    pub fn coerce(&'a self) -> Set {
        A::property_coerce(&self.value)
    }
}

pub trait PropertyValueCompare<'a, A> {
    fn property_compare(left: A, right: &Self) -> bool;
}

pub trait PropertyValueCoerce<'a, A> {
    fn property_coerce(value: &'a Self) -> A;
}

impl<'a, A> PropertyValueCompare<'a, A> for A
where
    A: PartialEq + 'a,
{
    fn property_compare(left: A, right: &A) -> bool {
        &left == right
    }
}

impl<'a, A> PropertyValueCoerce<'a, A> for A
where
    A: Clone + 'a,
{
    fn property_coerce(value: &'a A) -> A {
        value.clone()
    }
}

impl<'a, A> PropertyValueCompare<'a, A> for &'a A
where
    A: PartialEq + 'a,
{
    fn property_compare(left: A, right: &&A) -> bool {
        &left == *right
    }
}

impl<'a, A> PropertyValueCoerce<'a, A> for &'a A
where
    A: Clone + 'a,
{
    fn property_coerce(value: &'a &'a A) -> A {
        (*value).clone()
    }
}

impl<'a, A> PropertyValueCompare<'a, &'a A> for A
where
    A: PartialEq + 'a,
{
    fn property_compare(left: &A, right: &A) -> bool {
        left == right
    }
}

impl<'a, A> PropertyValueCoerce<'a, &'a A> for A
where
    A: 'a,
{
    fn property_coerce(value: &'a A) -> &'a A {
        value
    }
}

impl<'a> PropertyValueCompare<'a, &'a str> for String {
    fn property_compare(left: &str, right: &String) -> bool {
        left == right
    }
}

impl<'a> PropertyValueCoerce<'a, &'a str> for String {
    fn property_coerce(value: &String) -> &str {
        value.as_str()
    }
}

impl<'a> PropertyValueCompare<'a, Option<&'a str>> for String {
    fn property_compare(left: Option<&str>, right: &String) -> bool {
        if let Some(left) = left {
            left == right
        } else {
            false
        }
    }
}

impl<'a> PropertyValueCoerce<'a, Option<&'a str>> for String {
    fn property_coerce(value: &String) -> Option<&str> {
        Some(value.as_str())
    }
}

impl<'a> PropertyValueCompare<'a, Option<GString>> for String {
    fn property_compare(left: Option<GString>, right: &String) -> bool {
        if let Some(left) = left {
            left.as_str() == right
        } else {
            false
        }
    }
}

impl<'a> PropertyValueCoerce<'a, Option<GString>> for String {
    fn property_coerce(value: &'a String) -> Option<GString> {
        Some(value.to_owned().into())
    }
}

impl<'a> PropertyValueCompare<'a, Option<Widget>> for Image {
    fn property_compare(left: Option<Widget>, right: &Image) -> bool {
        if let Some(left) = left {
            if let Some(left) = left.downcast_ref::<Image>() {
                return left.get_property_icon_name() == right.get_property_icon_name()
                    && left.get_property_icon_size() == right.get_property_icon_size();
            }
        }
        false
    }
}

impl<'a> PropertyValueCoerce<'a, Option<&'a Image>> for Image {
    fn property_coerce(value: &'a Image) -> Option<&'a Image> {
        Some(value)
    }
}

impl PropertyValueCompare<'_, Vec<GString>> for &'_ [&'_ str] {
    fn property_compare(left: Vec<GString>, right: &&[&str]) -> bool {
        left == *right
    }
}

pub trait IntoPropertyValue<'a, A, Get, Set>
where
    A: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set> + 'a,
{
    fn into_property_value(self) -> PropertyValue<'a, A, Get, Set>;
}

impl<'a, A, Get, Set> IntoPropertyValue<'a, A, Get, Set> for A
where
    A: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set> + 'a,
{
    fn into_property_value(self) -> PropertyValue<'a, Self, Get, Set> {
        PropertyValue::new(self)
    }
}

impl<'a, A, Get, Set> IntoPropertyValue<'a, A, Get, Set> for &'a A
where
    A: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set> + Clone + 'a,
{
    fn into_property_value(self) -> PropertyValue<'a, A, Get, Set> {
        PropertyValue::new(self.clone())
    }
}

impl<'a, A, Get, Set> IntoPropertyValue<'a, Option<A>, Get, Set> for Option<&'a A>
where
    A: Clone,
    Option<A>: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set> + Clone + 'a,
{
    fn into_property_value(self) -> PropertyValue<'a, Option<A>, Get, Set> {
        PropertyValue::new(self.cloned())
    }
}

impl<'a, Get, Set> IntoPropertyValue<'a, String, Get, Set> for &'_ str
where
    String: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set>,
{
    fn into_property_value(self) -> PropertyValue<'a, String, Get, Set> {
        PropertyValue::new(self.to_string())
    }
}

impl<'a, Get, Set> IntoPropertyValue<'a, Image, Get, Set> for (&'_ str, IconSize)
where
    Image: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set>,
{
    fn into_property_value(self) -> PropertyValue<'a, Image, Get, Set> {
        let (name, size) = self;
        PropertyValue::new(Image::new_from_icon_name(Some(name), size))
    }
}

impl<'a, Get, Set> IntoPropertyValue<'a, Image, Get, Set> for &'_ str
where
    Image: PropertyValueCompare<'a, Get> + PropertyValueCoerce<'a, Set>,
{
    fn into_property_value(self) -> PropertyValue<'a, Image, Get, Set> {
        PropertyValue::new(Image::new_from_icon_name(Some(self), IconSize::Button))
    }
}
