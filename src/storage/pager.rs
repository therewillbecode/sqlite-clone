/*
    The pager is the part of sqlite that loads pointers to pages on disk into memory called the
    the page cache (aka buffer pool).

    The B-tree module requests information from the disk in fixed-size pages.
    The default page_size is 4096 bytes but can be any power of two between 512 and 65536 bytes.
    The B-tree driver requests particular pages from the page cache and notifies the page cache
    when it wants to modify pages or commit or rollback changes.

    A common question is "Hey this sounds like virtual memory. Why not just rely on the OS to manage files and map them
    into memory like with mmap and stop reinventing the wheel?"

    If we relied solely on the OS's virtual memory, then it would be up to the OS to decide which pages to remove from
    physical memory when our physical memory is full and we want to load another page. We want more control than this
    so in sqlite we build a "Pager" which sits a level of abstraction above the OS and works in tandem with the OS
    to give us more control over how pages are managed in physical memory.

    The OS is not your friend when it comes to databases. We want to use both the OS's page cache as well
    as our own sqlite page cache together as this boosts performance by removing unneeded system calls for disk I/O.

*/
