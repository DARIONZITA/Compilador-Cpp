#include <stdio.h>

int main()
{
    int i = 0;
    do
    {
        i = i + 1;
    } while (i < 10);
    do
    {
        i = i - 1;
    } while (i);
    do
    {
        i = 0;
    } while ("erro");
    return 0;
}
