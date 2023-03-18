import java.util.Scanner;

public class Sum {
  public static void main(String[] args) {
    final Scanner scanner = new Scanner(System.in);
    int sum = 0;
    for (int i = 0; i < 3; i += 1) {
      sum += scanner.nextInt();
    }
    System.out.print(sum);
  }
}
