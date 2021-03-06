extern crate winapi;
extern crate user32;
extern crate kernel32;

use winapi::*;
use user32::*;
use kernel32::*;

use std::ptr::{null_mut, null};
use std::mem::{size_of, uninitialized, transmute};

///////
// EXTERNS
///////

#[cfg(any(target_arch = "x86_64"))]
extern "system" {
	pub fn D2D1CreateFactory(
        factoryType: D2D1_FACTORY_TYPE,
		riid: REFIID, 
		pFactoryOptions: *const D2D1_FACTORY_OPTIONS,
        ppIFactory: *mut *mut ID2D1Factory
    ) -> HRESULT;
}

///////
// STRUCTURES
///////

pub struct MyAppResources{
    render_target: *mut ID2D1HwndRenderTarget,
    brush1: *mut ID2D1SolidColorBrush,
    brush2: *mut ID2D1SolidColorBrush
}

pub struct MyApp{
    resources: MyAppResources,
    factory: *mut ID2D1Factory,
    hwnd: HWND,
}


///////
// D2D1 SETUP
///////

/*
    Create a D2D1CreateFactory
*/
unsafe fn setup_d2d_factory(app: &mut MyApp){
    let null_options: *const D2D1_FACTORY_OPTIONS = null();
    let mut factory: *mut ID2D1Factory = null_mut();
    
    let result = D2D1CreateFactory(
        D2D1_FACTORY_TYPE_SINGLE_THREADED,
        &UuidOfID2D1Factory,
        null_options,
        &mut factory
    );
    
    if result != S_OK{
        panic!("Could not create D2D1 factory.");
    }
    
   app.factory = factory;
} 

/*
    Create the ressource used when drawing in the window.
    
*/
unsafe fn setup_d2d_resources(app: &mut MyApp){    
    
    //Check if the resources are already allocated.
    if !app.resources.render_target.is_null(){
        return;
    }else if app.factory.is_null(){
        panic!("Cannot initialize resources without a factory!");
    }
    
    let hwnd = app.hwnd;
	let mut rc: RECT = uninitialized();
    
    let mut resources = MyAppResources{
        render_target: null_mut(),
        brush1: null_mut(),
        brush2: null_mut(),
    };
    
    /*
        Structures for CreateHwndRenderTarget
    */
    GetClientRect(hwnd, &mut rc);
    let size = D2D_SIZE_U{width: (rc.right-rc.left) as u32,
				      height: (rc.bottom-rc.top) as u32};
    
    let pixel_format = D2D1_PIXEL_FORMAT{
        format: DXGI_FORMAT_B8G8R8A8_UNORM.0,
        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED
    };
    
    let render_props = D2D1_RENDER_TARGET_PROPERTIES{
        _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
        pixelFormat: pixel_format,
        dpiX: 0.0, dpiY: 0.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT
    };
    
    let hwnd_render_props = D2D1_HWND_RENDER_TARGET_PROPERTIES{
        hwnd: hwnd,
        pixelSize: size,
        presentOptions: D2D1_PRESENT_OPTIONS_NONE
    };
    
    /*
        Structures for ID2D1SolidColorBrush
    */
    let null_properties: *const D2D1_BRUSH_PROPERTIES = null();
    let gray = D2D1_COLOR_F{r: 0.345, g: 0.423, b: 0.463, a: 1.0};
    let red = D2D1_COLOR_F{r: 0.941, g: 0.353, b: 0.392, a: 1.0};
    
    /*
        Allocate the resources
    */

    let factory: &mut ID2D1Factory = &mut *app.factory;
    let mut rt: &mut ID2D1HwndRenderTarget;
    
    if factory.CreateHwndRenderTarget(&render_props, &hwnd_render_props, &mut resources.render_target) != S_OK{
        panic!("Could not create render target.");
    }
    
    rt = transmute(resources.render_target);
    
    if rt.CreateSolidColorBrush(&gray, null_properties, &mut resources.brush1) != S_OK{
        panic!("Could not create brush!");
    }
    
    if rt.CreateSolidColorBrush(&red, null_properties, &mut resources.brush2) != S_OK{
        panic!("Could not create brush!");
    }
    
    app.resources = resources;
}


/*
    Release the resources used by Direct2D
*/
unsafe fn clean_d2d_resources(app: &mut MyApp){
    if !app.resources.render_target.is_null(){
        (*app.resources.brush1).Release();
        (*app.resources.brush2).Release();
        (*app.resources.render_target).Release();
        
        app.resources.brush1 = null_mut();
        app.resources.brush2 = null_mut();
        app.resources.render_target = null_mut();
    }
}

/*
    Release the resources used by Direct2D
*/
unsafe fn clean_d2d(app: &mut MyApp){
    clean_d2d_resources(app);
    
    if !app.factory.is_null(){
        (*app.factory).Release();
        app.factory = null_mut();
    }
           
}

///////
// WINDOW PROCEDURE
///////

/*
    Painting event
*/
unsafe fn render_window(myapp: &mut MyApp) -> HRESULT{
    let identity = D2D1_MATRIX_3X2_F{
        matrix:[[1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0]]
    };
  
    let white = D2D1_COLOR_F{r:1.0, g:1.0, b:1.0, a:1.0};
    
    let render = &mut *myapp.resources.render_target;
    let mut render_size = D2D1_SIZE_F{width: 0.0, height: 0.0};
    
    render.BeginDraw();
    render.Clear(&white);
    
    render.SetTransform(&identity);
    render.GetSize(&mut render_size);

    
    // Draw a grid background.
    let mut count: FLOAT = 0.0;
    while count < render_size.width{
        render.DrawLine(
            D2D_POINT_2F{x: count, y: 0.0},
            D2D_POINT_2F{x: count, y: render_size.height},
            transmute(myapp.resources.brush1),
            0.5,
            null_mut()
        );
        
        count += 10.0;
    }
    
    count = 0.0;
    while count < render_size.height{
        render.DrawLine(
            D2D_POINT_2F{x: 0.0, y: count},
            D2D_POINT_2F{x: render_size.width, y: count},
            transmute(myapp.resources.brush1),
            0.5,
            null_mut()
        );
        
        count += 10.0;
    }
    
    // Draw two rectangles.
    let rx = render_size.width/2.0;
    let ry = render_size.height/2.0;
    let rect1 = D2D1_RECT_F{left: rx-50.0, right: rx+50.0, top: ry-50.0, bottom: ry+50.0};
    let rect2 = D2D1_RECT_F{left: rx-100.0, right: rx+100.0, top: ry-100.0, bottom: ry+100.0};
    
    render.FillRectangle(&rect1, transmute(myapp.resources.brush1));
    render.DrawRectangle(&rect2, transmute(myapp.resources.brush2), 3.0, null_mut());
    
    
    render.EndDraw(null_mut(), null_mut())
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT{
    let mut result: (LPARAM, bool) = (1, true);
    let myapp_ptr = GetWindowLongPtrW(hwnd, 0);
    let myapp: &mut MyApp = transmute(myapp_ptr);
   
    match msg{
        WM_PAINT =>{
            //Recreate the resources if the render target needs to be rebuilt
            setup_d2d_resources(myapp);
            
            // Render the window & check if the resources needs to be recreated.
            if render_window(myapp) == D2DERR_RECREATE_TARGET{
                clean_d2d_resources(myapp);
            }
        },
        WM_SIZE => {
          if myapp_ptr != 0{
            let width = GET_X_LPARAM(l) as u32;
            let height = GET_Y_LPARAM(l) as u32;
            let render_size = D2D_SIZE_U{width: width, height: height};
                    
            let render =  &mut *myapp.resources.render_target;
            render.Resize(&render_size);
          }else{
               result = (0, false);
          }
        },
        WM_DESTROY =>{
            PostQuitMessage(0);
        },
        WM_CREATE => {
            SetWindowLongPtrW(hwnd, 0, 0);
        },
        _ => {result = (0, false);}
    }

    match result.1{
        true => result.0,
        false => DefWindowProcW(hwnd, msg, w, l)
    }
}

///////
// WINDOW SETUP
///////

/*
    Create the window class.
*/
unsafe fn setup_class(class_name: &Vec<WCHAR>){
    let null_icon: HICON = null_mut();
    let null_background: HBRUSH = null_mut();
    let null_name: *const WCHAR = null();
    let module = GetModuleHandleW(null_name);
    
    let class =
    WNDCLASSEXW{
        cbSize: size_of::<WNDCLASSEXW>() as UINT,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc), 
        cbClsExtra: 0,
        cbWndExtra: 32,
        hInstance: module,
        hIcon: null_icon,
        hCursor: LoadCursorW(module, IDC_ARROW),
        hbrBackground: null_background,
        lpszMenuName: null_name,
        lpszClassName:  class_name.as_ptr(),
        hIconSm: null_icon
	};
    
    //Register the class
    match RegisterClassExW(&class){
        0 => panic!("Could not register class!"),
        _ => {}
    };
}

/*
    Create the window
*/
unsafe fn setup_window(app: &mut MyApp, class_name: &Vec<WCHAR>, window_name: &Vec<WCHAR>){
    let null_hwnd: HWND = null_mut();
    let null_menu: HMENU = null_mut();
    let null_name: *const WCHAR = null();
    let null: LPVOID = null_mut();
    let module = GetModuleHandleW(null_name);
    
    let hwnd = 
    CreateWindowExW(
        WS_EX_COMPOSITED,
        class_name.as_ptr(),
        window_name.as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT,
        600, 400,
        null_hwnd,
        null_menu,
        module,
        null
    );
    
    if hwnd.is_null(){
        panic!("Could not create window");
    }
    
    app.hwnd = hwnd;
}

/*
    Save the app address inside the window data.
*/
unsafe fn pack_app(app: &mut MyApp){
    SetWindowLongPtrW(app.hwnd, 0, transmute(app));
}



///////
// MAIN
///////

fn main() {
    unsafe{
        let mut app = MyApp{
            factory: null_mut(),
            hwnd: null_mut(),
            resources: MyAppResources{
                render_target: null_mut(),
                brush1: null_mut(),
                brush2: null_mut()
            }
         };
        
        // 'MyApp' as UTF16
        let class_name: Vec<WCHAR> = vec![77, 121, 65, 112, 112, 0];
        let window_name: Vec<WCHAR> = vec![77, 121, 65, 112, 112, 0];
       
        // Window setup
        setup_class(&class_name);
        setup_window(&mut app, &class_name, &window_name);
        pack_app(&mut app);
        
        // D2D1 Setup
        setup_d2d_factory(&mut app);
        setup_d2d_resources(&mut app);
        
        // Application Loop
        let mut msg = uninitialized();
        let null_handle: HWND = null_mut();
        while GetMessageW(&mut msg, null_handle, 0, 0) != 0{
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        //App cleaning
        clean_d2d(&mut app);
    }
    
}
