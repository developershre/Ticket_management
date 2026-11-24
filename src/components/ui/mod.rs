pub(crate) mod popup;

pub mod input;
#[allow(unused_imports)]
pub use input::Input;

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

pub mod clock_date_picker;
#[allow(unused_imports)]
pub use clock_date_picker::ClockDateTimePicker;

pub mod accordion;
#[allow(unused_imports)]
pub use accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};

pub mod alert;
#[allow(unused_imports)]
pub use alert::{Alert, AlertDescription, AlertTitle, AlertVariant};

pub mod alert_dialog;
#[allow(unused_imports)]
pub use alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger,
};

pub mod aspect_ratio;
#[allow(unused_imports)]
pub use aspect_ratio::AspectRatio;

pub mod breadcrumb;
#[allow(unused_imports)]
pub use breadcrumb::{
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList,
    BreadcrumbPage, BreadcrumbSeparator,
};

pub mod checkbox;
#[allow(unused_imports)]
pub use checkbox::Checkbox;

pub mod hover_card;
#[allow(unused_imports)]
pub use hover_card::{HoverCard, HoverCardContent, HoverCardTrigger};

pub mod input_otp;
#[allow(unused_imports)]
pub use input_otp::{InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot};

pub mod label;
#[allow(unused_imports)]
pub use label::Label;

pub mod menubar;
#[allow(unused_imports)]
pub use menubar::{
    Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarSeparator, MenubarShortcut,
    MenubarTrigger,
};

pub mod navigation_menu;
#[allow(unused_imports)]
pub use navigation_menu::{
    NavigationMenu, NavigationMenuContent, NavigationMenuItem, NavigationMenuLink,
    NavigationMenuList, NavigationMenuTrigger,
};

pub mod pagination;
#[allow(unused_imports)]
pub use pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationNext, PaginationPrevious,
};

pub mod popover;
#[allow(unused_imports)]
pub use popover::{Popover, PopoverClose, PopoverContent, PopoverTrigger};

pub mod progress;
#[allow(unused_imports)]
pub use progress::Progress;

pub mod radio_group;
#[allow(unused_imports)]
pub use radio_group::{RadioGroup, RadioGroupItem};

pub mod scroll_area;
#[allow(unused_imports)]
pub use scroll_area::{ScrollArea, ScrollBar};

pub mod sheet;
#[allow(unused_imports)]
pub use sheet::{
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetTitle,
    SheetTrigger,
};

pub mod skeleton;
#[allow(unused_imports)]
pub use skeleton::Skeleton;

pub mod slider;
#[allow(unused_imports)]
pub use slider::Slider;

pub mod table;
#[allow(unused_imports)]
pub use table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};

pub mod textarea;
#[allow(unused_imports)]
pub use textarea::Textarea;

pub mod toggle;
#[allow(unused_imports)]
pub use toggle::{Toggle, ToggleSize, ToggleVariant};

pub mod toggle_group;
#[allow(unused_imports)]
pub use toggle_group::{ToggleGroup, ToggleGroupItem};

#[cfg(test)]
mod tests;
