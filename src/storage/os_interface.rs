/*
    The SQLite OS Interface or virtual file system "VFS"
  
    Why not just use the OS to manage files? The OS is not your friend
    when it comes to databases. It just sees files being read and written 
    to, the OS doesn't know anything about how our database works
    and how it related to the things in that file. 
    
    Oh another reason is we want portability across OSs for our db. This module abstracts away
    operating system specific code for reading, writing and locking files.
*/