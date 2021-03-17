use lib::EasyScreen;

fn main(){
    let screen = EasyScreen::new();
    let mut c = 0;
    loop{
        c+=1;
        screen.fill(c);
    }
}

