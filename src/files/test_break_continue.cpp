#include <stdio.h>

int main()
{
    int i = 0;
    while (i < 10)
    {
        if (i == 5)
        {
            break;
        }
        i = i + 1;
    }
    for (i = 0; i < 10; i = i + 1)
    {
        if (i == 3)
        {
            continue;
        }
    }
    break;
    return 0;
}
