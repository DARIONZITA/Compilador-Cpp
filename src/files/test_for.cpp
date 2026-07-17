#include <stdio.h>

int main()
{
    int i;
    for (i = 0; i < 10; i = i + 1)
    {
        int x = 1;
    }
    for (i = 0; i; i = i + 1)
    {
        int y = 2;
    }
    for (i = 0; "erro"; i = i + 1)
    {
        int z = 3;
    }
    return 0;
}
