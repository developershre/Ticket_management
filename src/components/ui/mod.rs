pub(crate) mod popup;

pub mod input;
#[allow(unused_imports)]
pub use input::Input;

pub mod input_group;
#[allow(unused_imports)]
pub use input_group::{
    InputGroup, InputGroupAddon, InputGroupAddonPosition, InputGroupButton, InputGroupControl,
    InputGroupText,
};

pub mod dropdown_menu;
#[allow(unused_imports)]
pub use dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup,
    DropdownMenuItem, DropdownMenuLabel, DropdownMenuPortal, DropdownMenuRadioGroup,
    DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuSub,
    DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};

pub mod command;
#[allow(unused_imports)]
pub use command::{
    Command, CommandDialog, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList,
    CommandSeparator, CommandShortcut,
};

// ============================================================
// Shadcn Replicated UI Components
// ============================================================

pub mod dialog;
#[allow(unused_imports)]
pub use dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};

pub mod sidebar;
#[allow(unused_imports)]
pub use sidebar::{
    Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent,
    SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuAction, SidebarMenuBadge,
    SidebarMenuButton, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubButton, SidebarMenuSubItem,
    SidebarProvider, SidebarRail, SidebarSeparator, SidebarTrigger,
};

pub mod avatar;
#[allow(unused_imports)]
pub use avatar::{Avatar, AvatarFallback, AvatarImage};

pub mod calendar;
#[allow(unused_imports)]
pub use calendar::Calendar;

pub mod date_range_picker;
#[allow(unused_imports)]
pub use date_range_picker::DateRangePicker;

pub mod select;
#[allow(unused_imports)]
pub use select::{
    Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger,
    SelectValue, SimpleSelect,
};

pub mod item;
#[allow(unused_imports)]
pub use item::{Item, ItemActions, ItemContent, ItemDescription, ItemMedia, ItemTitle};

pub mod tooltip;
#[allow(unused_imports)]
pub use tooltip::{Tooltip, TooltipContent, TooltipProvider, TooltipTrigger};

pub mod switch;
#[allow(unused_imports)]
pub use switch::Switch;

pub mod breadcrumb;
#[allow(unused_imports)]
pub use breadcrumb::{
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator,
};

pub mod badge;
#[allow(unused_imports)]
pub use badge::Badge;

pub mod data_table;
#[allow(unused_imports)]
pub use data_table::{DataTable, DataTableColumn, DataTableFilter, SortValue};

pub mod collapsible;
#[allow(unused_imports)]
pub use collapsible::{Collapsible, CollapsibleContent, CollapsibleContext, CollapsibleTrigger};

pub mod tabs;
#[allow(unused_imports)]
pub use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};

pub mod context_menu;
#[allow(unused_imports)]
pub use context_menu::{
    ContextMenu, ContextMenuCheckboxItem, ContextMenuContent, ContextMenuGroup, ContextMenuItem,
    ContextMenuLabel, ContextMenuPortal, ContextMenuRadioGroup, ContextMenuRadioItem,
    ContextMenuSeparator, ContextMenuShortcut, ContextMenuSub, ContextMenuSubContent,
    ContextMenuSubTrigger, ContextMenuTrigger,
};

pub mod toast;
#[allow(unused_imports)]
pub use toast::{ToastProvider, ToastState, ToastVariant};

pub mod theme;
#[allow(unused_imports)]
pub use theme::{
    fetch_system_is_dark, use_theme, ThemeContext, ThemeMode, ThemeProvider, ThemeSegmented,
    ThemeToggle,
};

pub mod button;
#[allow(unused_imports)]
pub use button::{Button, ButtonSize, ButtonVariant};

pub mod card;
#[allow(unused_imports)]
pub use card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};

pub mod separator;
#[allow(unused_imports)]
pub use separator::Separator;

pub mod skeleton;
#[allow(unused_imports)]
pub use skeleton::Skeleton;

pub mod clock_date_picker;
#[allow(unused_imports)]
pub use clock_date_picker::ClockDateTimePicker;
